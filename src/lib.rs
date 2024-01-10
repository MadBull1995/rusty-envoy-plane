include!(concat!(env!("OUT_DIR"), "/mod.rs"));
pub mod cache;
// pub mod api;
pub mod resource;
pub mod server;
pub mod types;
pub mod well_knowns;

use cache::resource::ResponseType;
use cache::resources::ResourceType;
use cache::{
    snapshot::SnapshotCacheXds
};
use envoy::config::core::v3::{address, socket_address::PortSpecifier, Address, SocketAddress};
use envoy::extensions::filters::network::http_connection_manager::v3 as hcm;
use envoy::{
    // Config resources
    config::{
        cluster,
        endpoint::v3::Endpoint as EndpointPb,
        listener,
        listener::v3::{filter, Filter as FilterPb, Listener as ListenerPb},
    },
    service::cluster::v3::cluster_discovery_service_server::ClusterDiscoveryServiceServer,
    // xDS services
    service::listener::v3::listener_discovery_service_server::ListenerDiscoveryServiceServer,
    service::route::v3::route_discovery_service_server::RouteDiscoveryServiceServer,
};
use futures::FutureExt;
use google::protobuf::Any;
use prost::Message;
use resource::HCM_TYPE;
use server::service::Service;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::oneshot;
use tonic::transport::Server;
use types::config::cluster::Cluster;
use types::config::endpoint::Endpoint;
use types::config::listener::{Filter, FilterChain, Listener, ListenerFilterType};
use types::extensions::filters::network::HttpConnectionManager;
pub const ANY_PREFIX: &str = "type.googleapis.com";

pub use cache::{snapshot::Snapshot, simple::SnapshotCache};

pub fn run(left: usize, right: usize) -> usize {
    left + right
}

#[derive(Debug, Clone)]
pub struct ListenerBuilder {
    pub name: String,
    pub addr: Option<String>,
    pub port: Option<u32>,
    pub filter_chain: Option<FilterChain>,
}

impl ListenerBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            addr: None,
            filter_chain: None,
            port: None,
        }
    }

    pub fn addr(&mut self, addr: String) -> &mut Self {
        self.addr = Some(addr);
        self
    }

    pub fn filter_chain(&mut self, filter_chain: FilterChain) -> &mut Self {
        self.filter_chain = Some(filter_chain);
        self
    }

    pub fn port(&mut self, port: u32) -> &mut Self {
        self.port = Some(port);
        self
    }

    pub fn build(&self) -> Listener {
        let filter_chain = {
            if let Some(filter_chain) = self.filter_chain.clone() {
                filter_chain
            } else {
                panic!("must pass valid filter_chain to listener")
            }
        };
        Listener {
            name: self.name.clone(),
            addr: self.addr.clone().unwrap_or("0.0.0.0".to_string()),
            port: self.port.unwrap_or(5678),
            filter_chain: filter_chain,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ListenerFilterChainBuilder {
    name: Option<String>,
    filters: Vec<Filter>,
}

impl ListenerFilterChainBuilder {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            filters: Vec::new(),
        }
    }

    pub fn add_filter(&mut self, filter: Filter) -> &mut Self {
        self.filters.push(filter);

        self
    }

    pub fn build(&self) -> FilterChain {
        FilterChain {
            name: self.name.clone(),
            filters: self.filters.clone(),
        }
    }
}

/// Generate a filter chain wraps a set of match criteria, an option TLS context, a set of filters, and various other parameters.
#[derive(Clone, Debug)]
pub struct ListenerFilterBuilder {
    name: String,
    config_type: Option<ListenerFilterType>,
}

impl ListenerFilterBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            config_type: None,
        }
    }

    pub fn http_connection_manager(&mut self, hcm: HttpConnectionManager) -> &mut Self {
        self.config_type = Some(ListenerFilterType::HttpConnectionManager(hcm));

        self
    }

    pub fn build(&self) -> Filter {
        let cfg = {
            if let Some(config_type) = self.config_type.clone() {
                config_type
            } else {
                panic!("canot build listener filter without valid Any data");
            }
        };
        Filter {
            name: self.name.clone(),
            config_type: cfg,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClusterBuilder {
    name: String,
    hidden: bool,
    endpoints: Vec<Endpoint>,
    r#type: cluster::v3::cluster::DiscoveryType,
    lb_policy: cluster::v3::cluster::LbPolicy,
}

impl ClusterBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            hidden: false,
            endpoints: Vec::new(),
            lb_policy: cluster::v3::cluster::LbPolicy::RoundRobin,
            r#type: cluster::v3::cluster::DiscoveryType::StrictDns,
        }
    }

    pub fn hidden(&mut self, hide: bool) -> &mut Self {
        self.hidden = hide;

        self
    }

    pub fn add_endpoint(&mut self, endpoint: Endpoint) -> &mut Self {
        self.endpoints.push(endpoint);

        self
    }

    pub fn build(&self) -> Cluster {
        Cluster {
            name: self.name.clone(),
            endpoints: self.endpoints.clone(),
            hidden: false,
            lb_policy: self.lb_policy,
            r#type: self.r#type,
        }
    }
}

/// The main control plane data
pub struct EnvoyControlPlane {
    addr: &'static str,
    pub cache: Arc<SnapshotCacheXds<cache::IDHash>>,
    pub shutdown: Option<oneshot::Sender<()>>,
    snapshot_version: u64,
}

impl EnvoyControlPlane {
    pub fn new(identifier: &'static str, addr: &'static str) -> Self {
        let cache = Arc::new(SnapshotCacheXds::new(false, cache::IDHash, &identifier));

        Self {
            addr: addr,
            cache,
            shutdown: None,
            snapshot_version: 0
        }
    }

    fn new_snapshot_version(&mut self) -> String {
        self.snapshot_version+=1;
        format!("{}", self.snapshot_version)
    }

    pub async fn run(&mut self) {
        self.serve_with_shutdown().await;
    }

    pub fn new_snapshot(
        &mut self,
        resources: HashMap<ResponseType, Vec<ResourceType>>
    ) -> Result<Snapshot, Box<dyn Error + Send + Sync + 'static>> {
        let ver = self.new_snapshot_version();
        let snapshot = Snapshot::new(ver, resources)?;
        Ok(snapshot)
    }

    async fn serve_with_shutdown(&mut self) {
        let (tx, rx) = oneshot::channel::<()>();
        let addr = self.addr.parse().unwrap();

        let lds_service = Service::new(self.cache.clone());
        let cds_service = Service::new(self.cache.clone());
        let rds_service = Service::new(self.cache.clone());

        let lds = ListenerDiscoveryServiceServer::new(lds_service);
        let cds = ClusterDiscoveryServiceServer::new(cds_service);
        let rds = RouteDiscoveryServiceServer::new(rds_service);

        let server = Server::builder()
            .add_service(lds)
            .add_service(cds)
            .add_service(rds);

        // tokio::spawn(server.serve_with_shutdown(addr, rx.map(drop)));
        self.shutdown = Some(tx);
        server.serve(addr).await;
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        cache::{ConfigWatcher, Request},
        server::stream::StreamState,
    };

    use super::*;
    use tokio;
    #[tokio::test]

    async fn it_works() {
        let snapshot =
            cache::snapshot::SnapshotCacheXds::new(false, cache::IDHash, "my_control_plane");
        // dbg!(snapshot);
        let req = Request {
            node: Some(envoy::config::core::v3::Node {
                id: "some_node".to_string(),
                cluster: "cluster_name".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        // let (tx, rx) = tokio::sync::mpsc::channel(4);
        // let ss = StreamState::new(false, None);
        // let f = snapshot.create_watch(&req, &ss, tx).await;
        // let ns = snapshot.node_status().await;
        // dbg!(f);

        ()
    }
}
