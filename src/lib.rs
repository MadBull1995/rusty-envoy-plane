include!(concat!(env!("OUT_DIR"), "/mod.rs"));
pub mod cache;
// pub mod api;
pub mod resource;
pub mod well_knowns;
pub mod server;
pub mod types;

use envoy::config::endpoint::v3::Endpoint as EndpointPb;
use envoy::config::listener;
use envoy::config::listener::v3::{Listener as ListenerPb, Filter as FilterPb, filter};
use envoy::extensions::filters::network::http_connection_manager::v3 as hcm;
use envoy::config::core::v3::{address, Address, SocketAddress, socket_address::PortSpecifier};
use prost::Message;
use google::protobuf::Any;
use resource::HCM_TYPE;
use types::config::cluster::Cluster;
use types::config::listener::{Listener, Filter, ListenerFilterType, FilterChain};
use types::extensions::filters::network::HttpConnectionManager;
pub const ANY_PREFIX: &str = "type.googleapis.com";

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
            port: None
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
            filter_chain: filter_chain
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
    config_type: Option<ListenerFilterType>
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
            config_type: cfg
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClusterBuilder {
    name: String,
    hidden: bool,
}

impl ClusterBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            hidden: false,
        }
    }

    pub fn hidden(&mut self, hide: bool) -> &mut Self {
        self.hidden = hide;

        self
    }

    pub fn build(&self) -> Cluster {
        Cluster { 
            name: self.name.clone(),
            endpoints: Vec::new(),
            hidden: false,
        }
    }
}
// pub struct Listener {
//     pub name: String,
//     pub addr: String,
//     pub port: u32,
//     pub filters: Vec<Filter>
// }

// #[derive(Clone, Debug)]
// pub struct Route {
//     pub name: String,

// }

// #[derive(Clone, Debug)]
// pub struct Cluster {
//     pub name: String,
//     pub endpoints: Vec<Endpoint>,
//     pub hidden: bool,
// }

// #[derive(Clone, Debug)]
// pub struct Endpoint {
//     pub addr: String,
//     pub port: u32,
// }

// #[derive(Clone, Debug, PartialEq)]

// pub enum TypedConfig {
//     HttpConnectionManager(hcm::HttpConnectionManager),
// }

// impl TypedConfig {
//     pub fn into_any(&self) -> Any {
//        match self {
//         TypedConfig::HttpConnectionManager(hcm) => {
//             Any {
//                 type_url: HCM_TYPE.to_string(),
//                 value: hcm.encode_to_vec()
//             }
//         }
//        }
//     }
// }

// #[derive(Clone, Debug)]
// pub struct Filter {
//     pub name: String,
//     pub typed: TypedConfig,
// }

// impl Listener {
//     pub fn to_proto(&self) -> ListenerPb {
//         ListenerPb {
//             name: self.name.clone(),
//             address: Some(Address {
//                 address: Some(address::Address::SocketAddress(
//                     SocketAddress {
//                         address: self.addr.clone(),
//                         port_specifier: Some(PortSpecifier::PortValue(self.port)),
//                         ..Default::default()
//                     }
//                 ))
//             }),
//             filter_chains: self.filters_to_proto(),
//             ..Default::default()
//         }
//     }

//     fn filters_to_proto(&self) -> Vec<FilterChain> {
//         let mut filters = Vec::with_capacity(self.filters.len());
//         for filter in &self.filters {
//             filters.push(FilterPb {
//                 name: filter.name.clone(),
//                 config_type: Some(filter::ConfigType::TypedConfig(
//                     filter.typed.into_any()
//                 ))
//             })
//         }

//         vec![FilterChain {
//             filters,
//             ..Default::default()
//         }]
//     }
// }

#[cfg(test)]
mod tests {

    use crate::{cache::{ConfigWatcher, Request}, server::stream::StreamState};

    use super::*;
    use tokio;
    #[tokio::test]

    async fn it_works() {
        let snapshot = cache::snapshot::SnapshotCacheXds::new(false, cache::IDHash);
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
