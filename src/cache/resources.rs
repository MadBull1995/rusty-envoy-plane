use std::{collections::HashMap, borrow::Borrow};
use prost::Message;
use crate::google::protobuf::Any;
use crate::envoy::{
	extensions::transport_sockets::tls::v3 as tls,
	config::{
		cluster::v3 as cluster,
		endpoint::v3 as endpoint,
		route::v3 as route,
		listener::v3 as listener,
		bootstrap::v3 as bootstrap,
		core::v3 as core
	}, service::discovery
};
use super::{resource::get_resource_name, types::Resource};
use crate::resource::{ENDPOINT_TYPE, CLUSTER_TYPE, LISTENER_TYPE, ROUTE_TYPE, SECRET_TYPE, RUNTIME_TYPE, SCOPED_ROUTE_TYPE, EXTENSION_CONFIG_TYPE};
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceType {
	Cluster(cluster::Cluster),
    Endpoint(endpoint::ClusterLoadAssignment),
    Route(route::RouteConfiguration),
    Listener(listener::Listener),
    Secret(tls::Secret),
    Runtime(bootstrap::Runtime),
    ScopedRoute(route::ScopedRouteConfiguration),
    ExtensionConfig(core::TypedExtensionConfig),
	Resource(discovery::v3::Resource),
	Unknown,
}

impl ResourceType {
    pub fn into_any(&self) -> Any {
        match self {
            ResourceType::Cluster(cluster) => Any {
                type_url: CLUSTER_TYPE.to_string(),
                value: cluster.encode_to_vec(),
            },
            ResourceType::Endpoint(endpoint) => Any {
                type_url: ENDPOINT_TYPE.to_string(),
                value: endpoint.encode_to_vec(),
            },
            ResourceType::Route(route) => Any {
                type_url: ROUTE_TYPE.to_string(),
                value: route.encode_to_vec(),
            },
            ResourceType::Listener(listener) => Any {
                type_url: LISTENER_TYPE.to_string(),
                value: listener.encode_to_vec(),
            },
            ResourceType::Secret(secret) => Any {
                type_url: SECRET_TYPE.to_string(),
                value: secret.encode_to_vec(),
            },
            ResourceType::Runtime(runtime) => Any {
                type_url: RUNTIME_TYPE.to_string(),
                value: runtime.encode_to_vec(),
            },
            ResourceType::ScopedRoute(route) => Any {
                type_url: SCOPED_ROUTE_TYPE.to_string(),
                value: route.encode_to_vec(),
            },
            ResourceType::ExtensionConfig(config) => Any {
                type_url: EXTENSION_CONFIG_TYPE.to_string(),
                value: config.encode_to_vec(),
            },
            ResourceType::Resource(_) => todo!(),
            ResourceType::Unknown => todo!(),
        }
    }

    fn encode_to_vec(&self) -> Vec<u8> {
        match self {
            ResourceType::Cluster(cluster) => cluster.encode_to_vec(),
            ResourceType::Endpoint(endpoint) => endpoint.encode_to_vec(),
            ResourceType::Route(route) => route.encode_to_vec(),
            ResourceType::Listener(listener) => listener.encode_to_vec(),
            ResourceType::Secret(secret) => secret.encode_to_vec(),
            ResourceType::Runtime(runtime) => runtime.encode_to_vec(),
            ResourceType::ScopedRoute(route) => route.encode_to_vec(),
            ResourceType::ExtensionConfig(config) => config.encode_to_vec(),
            ResourceType::Resource(_) => todo!(),
            ResourceType::Unknown => todo!(),
        }
    }
}

impl Resource for cluster::Cluster {}
impl Resource for endpoint::ClusterLoadAssignment {}
impl Resource for route::RouteConfiguration {}
impl Resource for listener::Listener {}
impl Resource for tls::Secret {}
impl Resource for bootstrap::Runtime {}
impl Resource for route::ScopedRouteConfiguration {}
impl Resource for core::TypedExtensionConfig {}
impl Resource for discovery::v3::Resource {}

impl ResourceType {
	pub fn get_raw(&self) -> &dyn Resource {
		match self {
			ResourceType::Cluster(c) => c as &dyn Resource,
			ResourceType::Endpoint(_) => todo!(),
			ResourceType::Route(_) => todo!(),
			ResourceType::Listener(_) => todo!(),
			ResourceType::Secret(_) => todo!(),
			ResourceType::Runtime(_) => todo!(),
			ResourceType::ScopedRoute(_) => todo!(),
			ResourceType::ExtensionConfig(_) => todo!(),
			ResourceType::Unknown => todo!(),
    		ResourceType::Resource(_) => todo!(),
		}
	}
}

/// Resources is a versioned group of resources.
#[derive(Debug, Clone)]
pub struct Resources {
	/// Version information.
	pub(crate) version: String,

	/// Items in the group indexed by name.
    pub(crate) items: HashMap<String, ResourceType>,
}

// NewResources creates a new resource group.
pub fn new_resources(version: String, items: Vec<ResourceType>) -> Resources {
	let mut items_with_ttl = Vec::with_capacity(items.len());
	for i in items {
		items_with_ttl.push(i)	
	}
	return new_resources_with_ttl(version, items_with_ttl)
}

fn index_resources_by_name(items: Vec<ResourceType>) -> HashMap<String, ResourceType> {
	let mut indexed = HashMap::with_capacity(items.len());
	for item in items {
		indexed.insert(get_resource_name(&item), item);
	}

	indexed
}

fn new_resources_with_ttl(version: String, items:Vec<ResourceType>) -> Resources {
	Resources {
		version,
		items: index_resources_by_name(items)
	}
}