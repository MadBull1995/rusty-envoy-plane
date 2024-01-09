pub mod google {
    pub mod protobuf {
        tonic::include_proto!("google.protobuf");
    }
    pub mod rpc {
        tonic::include_proto!("google.rpc");
    }
}
pub mod envoy {
    pub mod config {
        pub mod accesslog {
            pub mod v3 {
                tonic::include_proto!("envoy.config.accesslog.v3");
            }
        }
        pub mod bootstrap {
            pub mod v3 {
                tonic::include_proto!("envoy.config.bootstrap.v3");
            }
        }
        pub mod cluster {
            pub mod v3 {
                tonic::include_proto!("envoy.config.cluster.v3");
            }
        }
        pub mod common {
            pub mod key_value {
                pub mod v3 {
                    tonic::include_proto!("envoy.config.common.key_value.v3");
                }
            }
            pub mod matcher {
                pub mod v3 {
                    tonic::include_proto!("envoy.config.common.matcher.v3");
                }
            }
            pub mod mutation_rules {
                pub mod v3 {
                    tonic::include_proto!("envoy.config.common.mutation_rules.v3");
                }
            }
        }
        pub mod core {
            pub mod v3 {
                tonic::include_proto!("envoy.config.core.v3");
            }
        }
        pub mod endpoint {
            pub mod v3 {
                tonic::include_proto!("envoy.config.endpoint.v3");
            }
        }
        pub mod grpc_credential {
            pub mod v3 {
                tonic::include_proto!("envoy.config.grpc_credential.v3");
            }
        }
        pub mod listener {
            pub mod v3 {
                tonic::include_proto!("envoy.config.listener.v3");
            }
        }
        pub mod metrics {
            pub mod v3 {
                tonic::include_proto!("envoy.config.metrics.v3");
            }
        }
        pub mod overload {
            pub mod v3 {
                tonic::include_proto!("envoy.config.overload.v3");
            }
        }
        pub mod ratelimit {
            pub mod v3 {
                tonic::include_proto!("envoy.config.ratelimit.v3");
            }
        }
        pub mod rbac {
            pub mod v3 {
                tonic::include_proto!("envoy.config.rbac.v3");
            }
        }
        pub mod route {
            pub mod v3 {
                tonic::include_proto!("envoy.config.route.v3");
            }
        }
        pub mod tap {
            pub mod v3 {
                tonic::include_proto!("envoy.config.tap.v3");
            }
        }
        pub mod trace {
            pub mod v3 {
                tonic::include_proto!("envoy.config.trace.v3");
            }
        }
        pub mod upstream {
            pub mod local_address_selector {
                pub mod v3 {
                    tonic::include_proto!("envoy.config.upstream.local_address_selector.v3");
                }
            }
        }
    }
    pub mod service {
        pub mod accesslog {
            pub mod v3 {
                tonic::include_proto!("envoy.service.accesslog.v3");
            }
        }
        pub mod auth {
            pub mod v3 {
                tonic::include_proto!("envoy.service.auth.v3");
            }
        }
        pub mod cluster {
            pub mod v3 {
                tonic::include_proto!("envoy.service.cluster.v3");
            }
        }
        pub mod discovery {
            pub mod v3 {
                tonic::include_proto!("envoy.service.discovery.v3");
            }
        }
        pub mod endpoint {
            pub mod v3 {
                tonic::include_proto!("envoy.service.endpoint.v3");
            }
        }
        pub mod event_reporting {
            pub mod v3 {
                tonic::include_proto!("envoy.service.event_reporting.v3");
            }
        }
        pub mod ext_proc {
            pub mod v3 {
                tonic::include_proto!("envoy.service.ext_proc.v3");
            }
        }
        pub mod extension {
            pub mod v3 {
                tonic::include_proto!("envoy.service.extension.v3");
            }
        }
        pub mod health {
            pub mod v3 {
                tonic::include_proto!("envoy.service.health.v3");
            }
        }
        pub mod listener {
            pub mod v3 {
                tonic::include_proto!("envoy.service.listener.v3");
            }
        }
        pub mod load_stats {
            pub mod v3 {
                tonic::include_proto!("envoy.service.load_stats.v3");
            }
        }
        pub mod metrics {
            pub mod v3 {
                tonic::include_proto!("envoy.service.metrics.v3");
            }
        }
        pub mod rate_limit_quota {
            pub mod v3 {
                tonic::include_proto!("envoy.service.rate_limit_quota.v3");
            }
        }
        pub mod ratelimit {
            pub mod v3 {
                tonic::include_proto!("envoy.service.ratelimit.v3");
            }
        }
        pub mod route {
            pub mod v3 {
                tonic::include_proto!("envoy.service.route.v3");
            }
        }
        pub mod runtime {
            pub mod v3 {
                tonic::include_proto!("envoy.service.runtime.v3");
            }
        }
        pub mod secret {
            pub mod v3 {
                tonic::include_proto!("envoy.service.secret.v3");
            }
        }
        pub mod status {
            pub mod v3 {
                tonic::include_proto!("envoy.service.status.v3");
            }
        }
        pub mod tap {
            pub mod v3 {
                tonic::include_proto!("envoy.service.tap.v3");
            }
        }
        pub mod trace {
            pub mod v3 {
                tonic::include_proto!("envoy.service.trace.v3");
            }
        }
    }
    pub mod extensions {
        pub mod filters {
            pub mod network {
                pub mod http_connection_manager {
                    pub mod v3 {
                        tonic::include_proto!(
                            "envoy.extensions.filters.network.http_connection_manager.v3"
                        );
                    }
                }
            }
        }
    }
}
