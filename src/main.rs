use std::{collections::HashMap, sync::Arc, time::Duration, vec};

use futures::future::FutureExt;
use rusty_envoy_plane::{EnvoyControlPlane, SnapshotCache, Snapshot};
use rusty_envoy_plane::envoy::config::cluster::v3 as clusterPb;
use rusty_envoy_plane::envoy::config::core::v3 as corePb;
use rusty_envoy_plane::envoy::config::listener::v3 as listenerPb;
use rusty_envoy_plane::{
    cache::{
        self,
        resources::ResourceType,
        snapshot::SnapshotCacheXds,
    },
    envoy::{
        config::{endpoint, route},
        extensions::filters::network::http_connection_manager,
        service::listener::v3::listener_discovery_service_server::ListenerDiscoveryServiceServer,
        service::cluster::v3::cluster_discovery_service_server::ClusterDiscoveryServiceServer,
    },
    server::service::Service,
    types::{
        config::{
            endpoint::Endpoint,
            listener::{Filter, ListenerFilterType},
            route::{Route, RouteConfiguration, RouteMatch, VirtualHost},
        },
        extensions::filters::network::{
            HttpConnectionManager, HttpFilter, HttpFilterType, RouteSpecifier,
        },
    },
    ClusterBuilder, ListenerBuilder, ListenerFilterBuilder, ListenerFilterChainBuilder,
};
use std::future::Future;
use tokio::sync::oneshot;
use tonic::transport::Server;

const XDS_ADDR: &str = "127.0.0.1:5678";

#[tokio::main]
pub async fn main() {
    println!("starting xds node...");
    let node_id = "test-id-test-cluster";
    let mut control_plane = EnvoyControlPlane::new(
        "my_control_plane",
        XDS_ADDR
    );
    let cloned_cache = control_plane.cache.clone();
    let mut resources = HashMap::with_capacity(10);
    let hcm = HttpConnectionManager {
        stat_prefix: "ingress_http".to_string(),
        codec_type: http_connection_manager::v3::http_connection_manager::CodecType::Auto,
        http_filters: vec![],
        server_name: None,
        route_specifier: RouteSpecifier::RouteConfig(RouteConfiguration {
            name: "local_route".to_string(),
            virtual_hosts: vec![VirtualHost {
                typed_per_filter_config: HashMap::new(),
                domains: vec!["*".to_string()],
                routes: vec![Route {
                    r#match: RouteMatch {
                        path_specifier: route::v3::route_match::PathSpecifier::Prefix(
                            "/".to_string(),
                        ),
                        case_sensitive: false,
                    },
                    action: route::v3::route::Action::Route(route::v3::RouteAction {
                        cluster_specifier: Some(
                            route::v3::route_action::ClusterSpecifier::Cluster(
                                "grpc_service".to_string(),
                            ),
                        ),
                        ..Default::default()
                    }),
                    name: "".to_string(),
                }],
                name: "service".to_string(),
            }],
        }),
    };
    let cluster = ClusterBuilder::new("grpc_service")
        .add_endpoint(Endpoint {
            cluster_name: "grpc_service".to_string(),
            lb_endpoints: vec![endpoint::v3::LbEndpoint {
                host_identifier: Some(endpoint::v3::lb_endpoint::HostIdentifier::Endpoint(
                    endpoint::v3::Endpoint {
                        address: Some(corePb::Address {
                            address: Some(corePb::address::Address::SocketAddress(
                                corePb::SocketAddress {
                                    address: "127.0.0.1".to_string(),
                                    port_specifier: Some(
                                        corePb::socket_address::PortSpecifier::PortValue(8080),
                                    ),
                                    ..Default::default()
                                },
                            )),
                        }),
                        ..Default::default()
                    },
                )),
                ..Default::default()
            }],
        })
        .build();
    let hcm_filter =
        ListenerFilterBuilder::new("envoy.filters.network.http_connection_manager")
            .http_connection_manager(hcm)
            .build();
    let filter_chain = ListenerFilterChainBuilder::new(None)
        .add_filter(hcm_filter)
        .build();
    let listener = ListenerBuilder::new("some_listener")
        .filter_chain(filter_chain)
        .port(9000)
        .build();
    let listener: listenerPb::Listener = listener.into();
    let cluster: clusterPb::Cluster = cluster.into();
    resources.insert(
        cache::resource::ResponseType::Listener,
        vec![ResourceType::Listener(listener)],
    );
    resources.insert(
        cache::resource::ResponseType::Cluster,
        vec![ResourceType::Cluster(cluster)],
    );

    let snapshot = control_plane.new_snapshot(resources).unwrap();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(20)).await;
        println!("setting snapshot 1");
        let _ = cloned_cache.set_snapshot(node_id, snapshot).await;
    });

    control_plane.run().await;
}
