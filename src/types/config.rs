use crate::types::extensions::filters::network::HttpConnectionManager;
use crate::google::protobuf::{Any, BoolValue};
use crate::envoy::{
    config::{route::v3 as routePb, listener::v3 as listenerPb, core::v3 as corePb, endpoint::v3 as endpointPb},

};

use prost::Message;
use super::ToProto;

pub mod cluster {
    use crate::envoy::config::cluster;

    use super::*;
    #[derive(Debug, Clone)]
    pub struct Cluster {
        pub name: String,
        pub endpoints: Vec<endpoint::Endpoint>,
        pub hidden: bool,
        pub r#type: cluster::v3::cluster::DiscoveryType,
        pub lb_policy: cluster::v3::cluster::LbPolicy,
    }

    impl Into<cluster::v3::Cluster> for Cluster {
        fn into(self) -> cluster::v3::Cluster {
            cluster::v3::Cluster {
                name: self.name.clone(),
                load_assignment: Some(endpointPb::ClusterLoadAssignment {
                    cluster_name: self.name.clone(),
                    endpoints: self.endpoints.into_iter().map(|e| Into::<endpointPb::LocalityLbEndpoints>::into(e)).collect(),
                    ..Default::default()
                }),
                cluster_discovery_type: Some(cluster::v3::cluster::ClusterDiscoveryType::r#Type(
                    self.r#type.into()
                )),
                lb_policy: self.lb_policy.into(),
                ..Default::default()
            }
        }
    }
}

pub mod core {

}

pub mod endpoint {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct Endpoint {
        pub cluster_name: String,
        pub lb_endpoints: Vec<endpointPb::LbEndpoint>,
    }

    impl Into<endpointPb::LocalityLbEndpoints> for Endpoint {
        fn into(self) -> endpointPb::LocalityLbEndpoints {
            endpointPb::LocalityLbEndpoints {
                lb_endpoints: self.lb_endpoints,
                ..Default::default()
            }
        }
    }
}

pub mod listener {

    use super::*;

    #[derive(Debug, Clone)]
    pub struct FilterChain {
        pub filters: Vec<Filter>,
        pub name: Option<String>,
    }

    impl Into<listenerPb::FilterChain> for FilterChain {
        fn into(self) -> listenerPb::FilterChain {
            listenerPb::FilterChain {
                filters: self.filters
                    .clone()
                    .into_iter()
                    .map(|f| Into::<listenerPb::Filter>::into(f)).collect(),
                name: self.name.clone().unwrap_or_default(),
                ..Default::default()
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum ListenerFilterType {
        HttpConnectionManager(HttpConnectionManager),
    }

    impl Into<Any> for ListenerFilterType {
        fn into(self) -> Any {
            match self {
                ListenerFilterType::HttpConnectionManager(hcm) => Any {
                    type_url: hcm.type_url().to_string(),
                    value: hcm.into_proto().encode_to_vec()
                }
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Listener {
        pub name: String,
        pub addr: String,
        pub port: u32,
        pub filter_chain: FilterChain
    }

    impl Into<listenerPb::Listener> for Listener {
        fn into(self) -> listenerPb::Listener {
            
            listenerPb::Listener {
                name: self.name.clone(),
                address: Some(corePb::Address {
                    address: Some(corePb::address::Address::SocketAddress(
                        corePb::SocketAddress {
                            address: self.addr.clone(),
                            port_specifier: Some(corePb::socket_address::PortSpecifier::PortValue(self.port)),
                            ..Default::default()
                        }
                    ))
                }),
                filter_chains: vec![
                    listenerPb::FilterChain {
                        filters: self.filter_chain.filters.into_iter().map(|f| Into::<listenerPb::Filter>::into(f)).collect(),
                        name: self.filter_chain.name.unwrap_or_default(),
                        ..Default::default()
                    }
                ],
                ..Default::default()
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Filter {
        pub name: String,
        pub config_type: ListenerFilterType,
    }

    impl Into<listenerPb::Filter> for Filter {
        fn into(self) -> listenerPb::Filter {
            let typed_config: Any = self.config_type.into();
            listenerPb::Filter {
                name: self.name,
                config_type: Some(listenerPb::filter::ConfigType::TypedConfig(typed_config))
            }
        }
    }
}

pub mod route {
    use std::collections::HashMap;

    use crate::resource::{VIRTUAL_HOST_TYPE, ROUTE_TYPE};

    use super::*;

    // #[derive(Debug, Clone)]
    // pub enum PathSpecifier {
        
    //     /// If specified, the route is a prefix rule meaning that the prefix must
    //     /// match the beginning of the ``:path`` header.
    //     Prefix(String),
        
    //     /// If specified, the route is an exact path rule meaning that the path must
    //     /// exactly match the ``:path`` header once the query string is removed.
    //     Path(String),

    //     /// If specified, the route is a regular expression rule meaning that the
    //     /// regex must match the ``:path`` header once the query string is removed. The entire path
    //     /// (without the query string) must match the regex. The rule will not match if only a
    //     /// subsequence of the ``:path`` header matches the regex.
    //     ///
    //     /// [#next-major-version: In the v3 API we should redo how path specification works such
    //     /// that we utilize StringMatcher, and additionally have consistent options around whether we
    //     /// strip query strings, do a case sensitive match, etc. In the interim it will be too disruptive
    //     /// to deprecate the existing options. We should even consider whether we want to do away with
    //     /// path_specifier entirely and just rely on a set of header matchers which can already match
    //     /// on :path, etc. The issue with that is it is unclear how to generically deal with query string
    //     /// stripping. This needs more thought.]
    //     SafeRegex, // TODO: type.matcher.v3.RegexMatcher

    //     /// If this is used as the matcher, the matcher will only match CONNECT or CONNECT-UDP requests.
    //     /// Note that this will not match other Extended CONNECT requests (WebSocket and the like) as
    //     /// they are normalized in Envoy as HTTP/1.1 style upgrades.
    //     /// This is the only way to match CONNECT requests for HTTP/1.1. For HTTP/2 and HTTP/3,
    //     /// where Extended CONNECT requests may have a path, the path matchers will work if
    //     /// there is a path present.
    //     /// Note that CONNECT support is currently considered alpha in Envoy.
    //     ConnectMatcher(routePb::route_match::ConnectMatcher),

    //     /// If specified, the route is a path-separated prefix rule meaning that the
    //     /// ``:path`` header (without the query string) must either exactly match the
    //     /// ``path_separated_prefix`` or have it as a prefix, followed by ``/``
    //     ///
    //     /// For example, ``/api/dev`` would match
    //     /// ``/api/dev``, ``/api/dev/``, ``/api/dev/v1``, and ``/api/dev?param=true``
    //     /// but would not match ``/api/developer``
    //     ///
    //     /// Expect the value to not contain ``?`` or ``#`` and not to end in ``/``
    //     PathSeparatedPrefix(String),

    //     /// [#extension-category: envoy.path.match]
    //     PathMatchPolicy {
    //         // TODO
    //     }
    // }


    #[derive(Debug, Clone)]
    pub struct RouteMatch {
        pub path_specifier: routePb::route_match::PathSpecifier,

        /// Indicates that prefix/path matching should be case sensitive. The default
        /// is true. Ignored for safe_regex matching.
        pub case_sensitive: bool,
    }

    impl Into<routePb::RouteMatch> for RouteMatch {
        fn into(self) -> routePb::RouteMatch {
            routePb::RouteMatch {
                path_specifier: Some(self.path_specifier.clone()),
                case_sensitive: Some(BoolValue { value: self.case_sensitive.clone() }),
                ..Default::default()
            }
        }
    }

    impl ToProto for RouteMatch {
        fn into_proto(&self) -> impl Message {
            routePb::RouteMatch {
                path_specifier: Some(self.path_specifier.clone()),
                case_sensitive: Some(BoolValue { value: self.case_sensitive.clone() }),
                ..Default::default()
            }
        }

        fn type_url(&self) -> &str {
            todo!()
        }
    }

    #[derive(Debug, Clone)]
    pub struct Route {
        pub name: String,
        pub r#match: RouteMatch,
        pub action: routePb::route::Action,
    }

    impl Into<routePb::Route> for Route {
        fn into(self) -> routePb::Route {
            let route_match: routePb::RouteMatch = self.r#match.into();
            routePb::Route {
                name: self.name,
                r#match: Some(route_match),
                action: Some(self.action),
                ..Default::default()
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct RouteAction {
        pub cluster: String,
    }

    impl FromIterator<Route> for Vec<routePb::Route> {
        fn from_iter<T: IntoIterator<Item = Route>>(iter: T) -> Self {
            let mut routes: Vec<routePb::Route> = Vec::new();

            for r in iter {
                let route: routePb::Route = r.into();
                routes.push(route);
            }

            routes
        }
    }

    impl ToProto for Route {
        fn into_proto(&self) -> impl Message {
            routePb::Route {
                ..Default::default()
            }
        }

        fn type_url(&self) -> &str {
            ROUTE_TYPE
        }
    }

    #[derive(Debug, Clone)]
    pub enum VirtualHostFilterType {
        Cors(routePb::CorsPolicy),
    }

    #[derive(Debug, Clone)]
    pub struct VirtualHost {
        pub name: String,
        pub domains: Vec<String>,
        pub routes: Vec<Route>,
        pub typed_per_filter_config: HashMap<String, Any>
    }

    impl Into<routePb::VirtualHost> for VirtualHost {
        fn into(self) -> routePb::VirtualHost {
            routePb::VirtualHost {
                name: self.name,
                domains: self.domains,
                routes: self.routes
                    .clone()
                    .into_iter()
                    .map(|r| Into::<routePb::Route>::into(r))
                    .collect(),
                ..Default::default()
            }
        }
    }

    impl ToProto for VirtualHost {
        fn into_proto(&self) -> impl Message {
            routePb::VirtualHost {
                name: self.name.clone(),
                domains: self.domains.clone(),
                routes: self.routes
                    .clone()
                    .into_iter()
                    .map(|r: Route| Into::<routePb::Route>::into(r))
                    .collect::<Vec<routePb::Route>>(),
                typed_per_filter_config: self.typed_per_filter_config.clone(),
                ..Default::default()
            }    
        }

        fn type_url(&self) -> &str {
            VIRTUAL_HOST_TYPE
        }
    }

    #[derive(Debug, Clone)]
    pub struct RouteConfiguration {
        pub name: String,
        pub virtual_hosts: Vec<VirtualHost>
    }

    impl Into<routePb::RouteConfiguration> for RouteConfiguration {
        fn into(self) -> routePb::RouteConfiguration {
            routePb::RouteConfiguration {
                name: self.name.clone(),
                virtual_hosts: self.virtual_hosts
                    .clone()
                    .into_iter()
                    .map(|vh| Into::<routePb::VirtualHost>::into(vh))
                    .collect(),
                ..Default::default()
            }
        }
    }

    impl ToProto for RouteConfiguration {
        fn into_proto(&self) -> impl prost::Message {
            routePb::RouteConfiguration {
                name: self.name.clone(),
                virtual_hosts: self.virtual_hosts
                    .clone()
                    .into_iter()
                    .map(|vh| vh.into())
                    .collect(),
                ..Default::default()
            }
        }

        fn type_url(&self) -> &str {
            todo!()
        }
    }
}