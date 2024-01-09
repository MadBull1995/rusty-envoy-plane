use std::{
    collections::HashMap,
    error::Error,
    sync::Arc,
};

use crate::{
    envoy::{config::core::v3 as core, service::discovery::v3 as discovery},
    google::protobuf,
    server::stream::StreamState,
};

use tokio::sync::Mutex;

pub mod order;
pub mod resource;
pub mod resources;
pub mod simple;
pub mod snapshot;
pub mod status;
pub mod types;
use self::{
    resource::get_resource_name,
    resources::ResourceType,
    types::{MarsheledResource, Resource, ResourceWithTTL},
};
use prost_types::{Any as pbAny, Duration};
use tokio::sync::mpsc;
pub trait NodeHash: Sync + Send + 'static {
    fn get_id(&self) -> String;
}

#[derive(Debug)]
pub struct IDHash;
impl IDHash {
    pub fn id(&self, node: Option<core::Node>) -> String {
        if let Some(n) = node {
            n.id
        } else {
            "".to_string()
        }
    }
}
impl NodeHash for IDHash {
    fn get_id(&self) -> String {
        self.id(None)
    }
}

impl NodeHash for core::Node {
    fn get_id(&self) -> String {
        format!("{}-{}", self.id, self.cluster)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct WatchId {
    pub node_id: String,
    pub index: u64,
}

pub trait ConfigWatcher: Sync + Send + 'static {
    fn create_watch(
        &self,
        request: &Request,
        stream_state: &Arc<Mutex<StreamState>>,
        response_channel: Arc<WatchResponder>,
    ) -> impl std::future::Future<Output = Option<WatchId>> + std::marker::Send;

    async fn create_delta_watch(
        &self,
        request: &DeltaRequest,
        response_channel: mpsc::Sender<DeltaResponse>,
    ) -> Option<WatchId>;
}
#[derive(Debug)]
pub enum FetchError {
    VersionUpToDate,
    NotFound,
}

pub trait ConfigFetcher: Sync + Send + 'static{
    async fn fetch<'a>(
        &'a self,
        request: &'a Request,
        type_url: &'static str,
    ) -> Result<Response, FetchError>;
}

pub trait Cache: ConfigWatcher + ConfigFetcher + Sync + Send + 'static {
    fn cancel_watch(
        &self,
        watch_id: &WatchId,
    ) -> impl std::future::Future<Output = ()> + std::marker::Send;
    async fn cancel_delta_watch(&self, watch_id: &WatchId);
}

pub trait ResponseTrait {
    // fn get_discovery_response(&mut self) -> Result<Arc<Response>, Box<dyn std::error::Error>>;
    fn get_request(&self) -> &Request;
    fn get_version(&self) -> Result<String, Box<dyn std::error::Error>>;
    // Omitting GetContext() equivalent
}

pub trait DeltaResponseTrait {
    fn get_delta_discovery_response(
        &self,
    ) -> Result<Arc<DeltaResponse>, Box<dyn std::error::Error>>;
    fn get_delta_request(&self) -> &DeltaRequest;
    fn get_system_version(&self) -> Result<String, Box<dyn std::error::Error>>;
    fn get_next_version_map(&self) -> &HashMap<String, String>;
    // Omitting GetContext() equivalent
}

#[derive(Debug, Clone)]
pub struct RawResponse {
    pub request: Request,
    pub version: String,
    pub resources: Vec<ResourceWithTTL>,
    pub heartbeat: bool,
    // Replacing Ctx with Rust idiomatic approach
    pub marshaled_response: Arc<Mutex<Option<Response>>>,
}

pub struct RawDeltaResponse {
    pub delta_request: DeltaRequest,
    pub system_version_info: String,
    pub resources: Vec<resources::ResourceType>, // Assuming Resource type is defined
    pub removed_resources: Vec<String>,
    pub next_version_map: HashMap<String, String>,
    // Replacing Ctx with Rust idiomatic approach
    pub marshaled_response: Arc<Mutex<Option<DeltaResponse>>>,
}

/// Request is an alias for the discovery request type.
pub type Request = discovery::DiscoveryRequest;

/// DeltaRequest is an alias for the delta discovery request type.
pub type DeltaRequest = discovery::DeltaDiscoveryRequest;

/// Response is an alias for the discovery request type.
pub type Response = discovery::DiscoveryResponse;

/// DeltaResponse is an alias for the delta discovery response type.
pub type DeltaResponse = discovery::DeltaDiscoveryResponse;

pub type WatchResponse = (Request, Response);

pub type WatchResponder = mpsc::Sender<WatchResponse>;

const DELTA_RESOURCE_TYPE_URL: &str = "type.googleapis.com/envoy.service.discovery.v3.Resource";

impl RawResponse {
    pub fn maybe_create_ttl_resource(
        &self,
        r: ResourceWithTTL,
    ) -> Result<(ResourceType, String), Box<dyn Error  + Send + Sync + 'static>> {
        if let Some(ttl) = r.ttl {
            let mut wrapped_resource = discovery::Resource {
                name: get_resource_name(&r.resource),
                ttl: Some(protobuf::Duration {
                    seconds: ttl.as_secs() as i64,
                    nanos: 0,
                }),
                ..Default::default()
            };

            if !self.heartbeat {
                match r.resource {
                    msg => {
                        // let rsc: Result<pbAny, prost::EncodeError> = pbAny::from_msg(&msg.get_raw());
                        wrapped_resource.resource = Some(protobuf::Any {
                            type_url: DELTA_RESOURCE_TYPE_URL.to_string(),
                            value: marshal_resource_type(&msg).unwrap(),
                        });
                    }
                }
            }
            Ok((
                ResourceType::Resource(wrapped_resource),
                DELTA_RESOURCE_TYPE_URL.to_string(),
            ))
        } else {
            Ok((r.resource, self.get_request().type_url.clone()))
        }
    }
}

impl ResponseTrait for RawResponse {
    // fn get_discovery_response(&mut self) -> Result<Arc<Response>, Box<dyn std::error::Error>> {
    //     // Obtain a lock on the marshaled response
    //     let mut marshaled_response_lock = self.marshaled_response.lock().await;
    //     // Check if the response is already marshaled and cached
    //     if let Some(response) = marshaled_response_lock {
    //         // If already marshaled, return a clone of the Arc
    //         return Ok(Arc::new(response.clone()));
    //     }

    //     // If not marshaled, marshal the resources and create the response
    //     let mut marshaled_resources = Vec::new();

    //     for r in &self.resources {
    //         let (maybe_ttld_resource, resource_type) = self.maybe_create_ttl_resource(r.clone())?;
    //         let marshaled_resource = marshal_resource_type(&maybe_ttld_resource)?;
    //         marshaled_resources.push(protobuf::Any {
    //             type_url: resource_type,
    //             value: marshaled_resource,
    //         });
    //     }

    //     let response = discovery::DiscoveryResponse {
    //         version_info: self.version.clone(),
    //         resources: marshaled_resources,
    //         type_url: self.get_request().type_url.clone(),
    //         ..Default::default()
    //     };

    //     // Cache the marshaled response in an Arc
    //     self.marshaled_response = Some(response.clone());

    //     // Return a clone of the Arc
    //     Ok(Arc::new(response))
    // }

    fn get_request(&self) -> &Request {
        todo!()
    }

    fn get_version(&self) -> Result<String, Box<dyn std::error::Error>> {
        todo!()
    }
}

fn marshal_resource_type(
    resource_type: &ResourceType,
) -> Result<Vec<u8>, Box<dyn std::error::Error  + Send + Sync + 'static>> {
    match resource_type {
        ResourceType::Cluster(cluster) => marshal_resource(cluster),
        ResourceType::Endpoint(endpoint) => marshal_resource(endpoint),
        ResourceType::Route(route) => marshal_resource(route),
        ResourceType::Listener(listener) => marshal_resource(listener),
        ResourceType::Secret(secret) => marshal_resource(secret),
        ResourceType::Runtime(runtime) => marshal_resource(runtime),
        ResourceType::ScopedRoute(scoped_route) => marshal_resource(scoped_route),
        ResourceType::ExtensionConfig(extension_config) => marshal_resource(extension_config),
        ResourceType::Resource(resource) => marshal_resource(resource),
        ResourceType::Unknown => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unknown resource type",
        ))),
    }
}

fn marshal_resource<R: Resource>(resource: &R) -> Result<MarsheledResource, Box<dyn Error  + Send + Sync + 'static>> {
    let mut buf = Vec::new();
    resource.encode(&mut buf)?;
    Ok(buf)
}

impl DeltaResponseTrait for RawDeltaResponse {
    fn get_delta_discovery_response(
        &self,
    ) -> Result<Arc<DeltaResponse>, Box<dyn std::error::Error>> {
        todo!()
    }

    fn get_delta_request(&self) -> &DeltaRequest {
        todo!()
    }

    fn get_system_version(&self) -> Result<String, Box<dyn std::error::Error>> {
        todo!()
    }

    fn get_next_version_map(&self) -> &HashMap<String, String> {
        todo!()
    }
}
