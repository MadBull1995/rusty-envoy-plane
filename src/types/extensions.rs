use crate::google::protobuf::Any;
use crate::envoy::{
    extensions::{
        filters::{
            network::{
                http_connection_manager::v3 as hcm,
            },
            http::{
                cors::v3 as cors,
                router::v3 as router,
                grpc_web::v3 as grpc_web
            },
        },
        
    },
    config::route::v3 as route,
};

pub mod filters {
    pub mod network {
        use prost::Message;

        use crate::{types::{ToProto, config::route::RouteConfiguration}, resource::{HCM_TYPE, GRPC_WEB_TYPE, ROUTE_TYPE}};

        use super::super::*;

        #[derive(Debug, Clone)]
        pub enum RouteSpecifier {
            Rds(hcm::Rds),
            RouteConfig(RouteConfiguration),
            ScopedRoutes(hcm::ScopedRoutes),
        }

        impl Into<Any> for RouteSpecifier {
            fn into(self) -> Any {
                match self {
                    RouteSpecifier::RouteConfig(route_configuration) => Any {
                        type_url: ROUTE_TYPE.to_string(),
                        value: route_configuration.into_proto().encode_to_vec()
                    },
                    RouteSpecifier::Rds(_) => todo!(),
                    RouteSpecifier::ScopedRoutes(_) => todo!()
                }
            }
        }

        

        #[derive(Debug, Clone)]
        pub enum HttpFilterType {
            Cors {
                cors: cors::Cors,
                cors_policy: cors::CorsPolicy,
            },
            Router(router::Router),
            GrpcWeb(grpc_web::GrpcWeb),
        }

        impl Into<Any> for HttpFilterType {
            fn into(self) -> Any {
                match self {
                    HttpFilterType::Cors { cors, cors_policy } => todo!(),
                    HttpFilterType::GrpcWeb(grpc_web) => Any {
                        type_url: GRPC_WEB_TYPE.to_string(),
                        value: grpc_web.encode_to_vec()
                    },
                    HttpFilterType::Router(_) => todo!(),
                }
            }
        }

        #[derive(Debug, Clone)]
        pub struct HttpFilter {
            pub name: String,
            pub config_type: HttpFilterType,
        }

        impl Into<hcm::HttpFilter> for HttpFilter {
            fn into(self) -> hcm::HttpFilter {
                let typed_config: Any = self.config_type.into();
                hcm::HttpFilter {
                    name: self.name,
                    config_type: Some(hcm::http_filter::ConfigType::TypedConfig(typed_config)),
                    ..Default::default()

                }
            }
        }

        #[derive(Debug, Clone)]
        pub struct HttpConnectionManager {
            pub codec_type: hcm::http_connection_manager::CodecType,
            pub stat_prefix: String,
            pub route_specifier: RouteSpecifier,
            pub http_filters: Vec<HttpFilter>,
            pub server_name: Option<String>,
            // pub access_log: Vec<>

        }

        impl ToProto for HttpConnectionManager {
            fn into_proto(&self) -> impl Message {
                let mut route_config: route::RouteConfiguration = route::RouteConfiguration {
                    ..Default::default()
                };
                match &self.route_specifier {
                    RouteSpecifier::RouteConfig(rc) => {
                        route_config = Into::<route::RouteConfiguration>::into(rc.clone());
                    },
                    _ => todo!()
                }
                hcm::HttpConnectionManager {
                    codec_type: self.codec_type.into(),
                    stat_prefix: self.stat_prefix.clone(),
                    route_specifier: Some(hcm::http_connection_manager::RouteSpecifier::RouteConfig(
                        route_config
                    )),
                    http_filters: self.http_filters
                        .clone()
                        .into_iter()
                        .map(|hf| Into::<hcm::HttpFilter>::into(hf))
                        .collect(),
                    server_name: self.server_name.clone().unwrap_or("".to_string()),
                    ..Default::default()
                }
            }
            fn type_url(&self) -> &str {
                HCM_TYPE
            }
        }
    }
}