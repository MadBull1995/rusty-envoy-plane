use std::{any::Any, collections::{HashMap, HashSet}, error::Error};

use prost::Message;

use crate::{
    ANY_PREFIX,
    resource,
    envoy::{
        extensions::filters::network::http_connection_manager::v3::{self as hcm, HttpConnectionManager},
        config::{
            endpoint::v3 as endpoint,
            cluster::v3 as cluster,
            listener::v3 as listener,
        }
    }, well_knowns, google::protobuf as pb, envoy::extensions::filters::network::http_connection_manager::v3::{http_connection_manager::RouteSpecifier, scoped_routes::ConfigSpecifier}
};

use super::{resources::{Resources, ResourceType}, types::{ResourceWithName, Resource}};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ResponseType {
    Endpoint,
    Cluster,
    Route,
    ScopedRoute,
    VirtualHost,
    Listener,
    Secret,
    Runtime,
    ExtensionConfig,
    RateLimitConfig,
    UnknownType,
}

impl ResponseType {
    pub fn from_idx(idx: usize) -> Self {
        match idx {
            0 => ResponseType::Cluster,
            1 => ResponseType::Endpoint,
            2 => ResponseType::Listener,
            3 => ResponseType::Route,
            4 => ResponseType::ScopedRoute,
            5 => ResponseType::VirtualHost,
            6 => ResponseType::Secret,
            7 => ResponseType::Runtime,
            8 => ResponseType::ExtensionConfig,
            9 => ResponseType::RateLimitConfig,
            _ => ResponseType::UnknownType,
        }
    }
}

pub fn get_response_type(type_url: &str) -> ResponseType {
    match type_url {
        resource::ENDPOINT_TYPE => ResponseType::Endpoint,
        resource::CLUSTER_TYPE => ResponseType::Cluster,
        resource::ROUTE_TYPE => ResponseType::Route,
        resource::SCOPED_ROUTE_TYPE => ResponseType::ScopedRoute,
        resource::VIRTUAL_HOST_TYPE => ResponseType::VirtualHost,
        resource::LISTENER_TYPE => ResponseType::Listener,
        resource::SECRET_TYPE => ResponseType::Secret,
        resource::RUNTIME_TYPE => ResponseType::Runtime,
        resource::EXTENSION_CONFIG_TYPE => ResponseType::ExtensionConfig,
        resource::RATE_LIMIT_CONFIG_TYPE => ResponseType::RateLimitConfig,
        _ => ResponseType::UnknownType,
    }
}

pub fn get_response_type_url(response_type: ResponseType) -> Result<String, Box<dyn Error  + Send + Sync + 'static>> {
    match response_type {
        ResponseType::Cluster => Ok(resource::CLUSTER_TYPE.to_string()),
        ResponseType::Endpoint => Ok(resource::ENDPOINT_TYPE.to_string()),
        ResponseType::Route => Ok(resource::ROUTE_TYPE.to_string()),
        ResponseType::ScopedRoute => Ok(resource::SCOPED_ROUTE_TYPE.to_string()),
        ResponseType::VirtualHost => Ok(resource::VIRTUAL_HOST_TYPE.to_string()),
        ResponseType::Listener => Ok(resource::LISTENER_TYPE.to_string()),
        ResponseType::Secret => Ok(resource::SECRET_TYPE.to_string()),
        ResponseType::Runtime => Ok(resource::RUNTIME_TYPE.to_string()),
        ResponseType::ExtensionConfig => Ok(resource::EXTENSION_CONFIG_TYPE.to_string()),
        ResponseType::RateLimitConfig => Ok(resource::RATE_LIMIT_CONFIG_TYPE.to_string()),
        ResponseType::UnknownType => Err("Unknown response type".into()),
    }
}

pub fn get_resource_name(res: &ResourceType) -> String {
    match res {
        ResourceType::Cluster(c) => {
            c.name.clone()
        },
        _ => "".to_string()
    }
}

pub fn get_resource_names(resources: &[ResourceType]) -> Vec<String> {
    resources.iter().map(|res| get_resource_name(res)).collect()
}

pub fn get_all_resource_references<'a>(resource_groups: &'a HashMap<ResponseType, Resources>) -> HashMap<ResponseType, HashMap<String, bool>> {
    let mut ret = HashMap::new();

    let response_types_with_references = vec![
        ResponseType::Cluster,
        ResponseType::Listener,
        ResponseType::ScopedRoute,
        // ... other types that have references
    ].into_iter().collect::<HashSet<_>>();

    for (response_type, resource_group) in resource_groups {
        if response_types_with_references.contains(&response_type) {
            get_resource_references(&resource_group.items, &mut ret);
        }
    }

    ret
}

fn get_listener_references(src: &listener::Listener, out: &mut HashMap<ResponseType, HashMap<String, bool>> ) {
    let mut routes = HashMap::new();
    
    for chain in &src.filter_chains {
        for filter in &chain.filters {
            if let Some(config) = get_http_connection_manager(filter) {
                if let Some(route_specifier) = config.route_specifier {
                    match route_specifier {
                        RouteSpecifier::Rds(rds) => {
                            // If using RDS, add the refernced route map.
                            if !rds.route_config_name.is_empty() {
                                routes.insert(rds.route_config_name.clone(), true);
                            }
                        },
                        RouteSpecifier::ScopedRoutes(scoped_routes) => {
                            // If the scoped route mapping is embedded, add the referenced route resource names.
                            if let Some(config_specifier) = &scoped_routes.config_specifier {
                                match config_specifier {
                                    ConfigSpecifier::ScopedRouteConfigurationsList(scoped_route_configs_list) => {
                                        for s in &scoped_route_configs_list.scoped_route_configurations {
                                            routes.insert(s.route_configuration_name.clone(), true);
                                        }
                                    }
                                    ConfigSpecifier::ScopedRds(_) => todo!(),
                                }
                            }
                        },
                        RouteSpecifier::RouteConfig(_) => todo!(),
                    }
                } 
            }
        }
    }

    if !routes.is_empty() {
        out.entry(ResponseType::Route)
           .or_insert_with(HashMap::new)
           .extend(routes);
    }
}

// Placeholder for getting the HTTP connection manager from a filter
// You will need to implement this based on your actual data structure.
fn get_http_connection_manager(filter: &listener::Filter) -> Option<hcm::HttpConnectionManager> {
    // Implement logic to extract HTTP connection manager configuration from the filter
    match &filter.config_type {
        Some(listener::filter::ConfigType::TypedConfig(any)) => {
            // Extract the HttpConnectionManager from the Any type
            deserialize_any_to_http_connection_manager(any)
        }
        Some(listener::filter::ConfigType::ConfigDiscovery(config_source)) => {
            // Handle the case where the configuration comes from an extension config source
            // Implement your logic based on how you want to handle ConfigDiscovery
            // ...
            None // Placeholder
        }
        _ => None, // In case of no config or unknown config
    }
}

fn deserialize_any_to_http_connection_manager(any: &pb::Any) -> Option<hcm::HttpConnectionManager> {
    // Check if the type URL is for HttpConnectionManager
    if any.type_url == well_knowns::HTTP_CONNECTION_MANAGER.to_string() {
        // Attempt to deserialize the bytes into HttpConnectionManager
        hcm::HttpConnectionManager::decode(any.value.as_slice()).ok()
        // Some()
    } else {
        // If the type URL does not match, return None
        None
    }
}

fn get_cluster_references(src: &cluster::Cluster, out: &mut HashMap<ResponseType, HashMap<String, bool>>) {
    let mut endpoints = HashMap::new();
    
    if let Some(cluster_discovery_type) = src.cluster_discovery_type.clone() {
        match cluster_discovery_type {
            cluster::cluster::ClusterDiscoveryType::Type(t) => {
                match t {
                    3 => {
                        if let Some(eds_cluster_config) = &src.eds_cluster_config {
                            if eds_cluster_config.service_name != "" {
                                endpoints.insert(eds_cluster_config.service_name.clone(), true);
                            } else {
                                endpoints.insert(src.name.clone(), true);
                            }
                        } else {
                            endpoints.insert(src.name.clone(), true);
                        }
                    },
                    _ => unreachable!()
                }
            }
            cluster::cluster::ClusterDiscoveryType::ClusterType(_) => unreachable!(),
        }
    }

    if !endpoints.is_empty() {
        out.entry(ResponseType::Endpoint)
           .or_insert_with(HashMap::new)
           .extend(endpoints);
    }
}

pub fn get_resource_references(
    resources: &HashMap<String, ResourceType>,
    out: &mut HashMap<ResponseType, HashMap<String, bool>>,
) {
    for (_, res) in resources.iter() {
        match res {
            ResourceType::Cluster(c) => {
                get_cluster_references(c, out);    
            },
            _ => todo!()
        }
    }
}