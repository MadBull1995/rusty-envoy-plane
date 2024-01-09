use std::{sync::Arc, collections::HashMap, vec, time::Duration};

use rusty_envoy_plane::{
    envoy::{service::listener::v3::listener_discovery_service_server::ListenerDiscoveryServiceServer, config::route, extensions::filters::network::http_connection_manager},
    server::service::Service,
    cache::{snapshot::{SnapshotCacheXds, Snapshot}, self, simple::SnapshotCache, resources::ResourceType}, ListenerBuilder, types::{config::{listener::{Filter, ListenerFilterType}, route::{RouteConfiguration, VirtualHost, Route, RouteMatch}}, extensions::filters::network::{HttpConnectionManager, RouteSpecifier, HttpFilter, HttpFilterType}}, ListenerFilterBuilder, ListenerFilterChainBuilder, ClusterBuilder
};
use rusty_envoy_plane::envoy::config::cluster::v3 as clusterPb;
use rusty_envoy_plane::envoy::config::listener::v3 as listenerPb;
use std::future::Future;
use futures::future::FutureExt;
use tokio::sync::oneshot;
use tonic::transport::Server;

const XDS_ADDR: &str = "127.0.0.1:5678";

pub struct XdsControlPlane {
    addr: String,
    cache: Arc<SnapshotCacheXds<cache::IDHash>>,
    shutdown: Option<oneshot::Sender<()>>,
}

impl XdsControlPlane {
    pub fn new() -> Self {
        let cache = Arc::new(SnapshotCacheXds::new(false, cache::IDHash));

        Self {
            addr: XDS_ADDR.to_string(),
            cache,
            shutdown: None,
        }
    }

    pub async fn run(&mut self)
    {
        self.serve_with_shutdown().await;
    }

    async fn serve_with_shutdown(&mut self) {
        let (tx, rx) = oneshot::channel::<()>();
        let addr = self.addr.parse().unwrap();
        let lds_service = Service::new(self.cache.clone());
        let lds = ListenerDiscoveryServiceServer::new(lds_service);
        let server = Server::builder()
            .add_service(lds);
        
        // tokio::spawn();
        // self.shutdown = Some(tx);
        server.serve(addr).await;
    }
}

#[tokio::main]
pub async fn main() {
    println!("starting xds node...");
    let mut node = XdsControlPlane::new();
    let node_id = "test-id-test-cluster";
    let mut resources = HashMap::new();
    let hcm = HttpConnectionManager {
        stat_prefix: "ingress_http".to_string(),
        codec_type: http_connection_manager::v3::http_connection_manager::CodecType::Auto,
        http_filters: vec![
            
        ],
        server_name: None,
        route_specifier: RouteSpecifier::RouteConfig(RouteConfiguration {
            name: "local_route".to_string(),
            virtual_hosts: vec![
                VirtualHost {
                    typed_per_filter_config: HashMap::new(),
                    domains: vec!["*".to_string()],
                    routes: vec![
                        Route {
                            r#match: RouteMatch {
                                path_specifier: route::v3::route_match::PathSpecifier::Prefix("/".to_string()),
                                case_sensitive: false,
                            },
                            action: route::v3::route::Action::Route(route::v3::RouteAction {
                                cluster_specifier: Some(route::v3::route_action::ClusterSpecifier::Cluster("grpc_service".to_string())),
                                ..Default::default()
                            }),
                            name: "".to_string()
                        }
                    ],
                    name: "service".to_string(),
                }
            ],
        })
    };
    let http_connection_manager = ListenerFilterBuilder::new("envoy.filters.network.http_connection_manager")
        .http_connection_manager(hcm).build();
    let filter_chain = ListenerFilterChainBuilder::new(None)
        .add_filter(http_connection_manager).build();
    let listener = ListenerBuilder::new("some_listener")
        .filter_chain(filter_chain).build();
    // let listener = Listener {
    //     name: "some_listener".to_string(),
    //     addr: "0.0.0.0".to_string(),
    //     port: 9000,
    //     filters: vec![
    //         Filter {
    //             name: "envoy.filters.network.http_connection_manager".to_string(),
    //             typed: rusty_envoy_plane::TypedConfig::HttpConnectionManager(http_connection_manager::v3::HttpConnectionManager {
    //                 stat_prefix: "ingress_http".to_string(),
    //                 route_specifier: Route {

    //                 }
    //                 ..Default::default()
    //             })
    //         }
    //     ]
    // };
    let cluster = ClusterBuilder::new("grpc_service").build();


    let listener: listenerPb::Listener = listener.into();
    let cluster: clusterPb::Cluster = cluster.into();
    resources.insert(cache::resource::ResponseType::Listener, vec![
        ResourceType::Listener(listener)
    ]);
    resources.insert(cache::resource::ResponseType::Cluster, vec![
        ResourceType::Cluster(cluster)
    ]);
    let snapshot = Snapshot::new("1".to_string(), resources.clone()).unwrap();
    let cloned= node.cache.clone();
    let snapshot_1 = Snapshot::new("2".to_string(), resources).unwrap();
    
    tokio::spawn(async move { 
        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("setting snapshot 1");
        cloned.set_snapshot(node_id, snapshot.clone()).await;
        tokio::time::sleep(Duration::from_secs(10)).await;
        println!("setting snapshot 2");
        cloned.set_snapshot(node_id, snapshot_1).await;
    });
    node.run().await;
}