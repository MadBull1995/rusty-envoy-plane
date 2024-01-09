use crate::envoy::config::listener::v3 as listener;
// use crate::api::envoy::::listener::v3 as listener;

pub const API_TYPE_PREFIX: &str = "type.googleapis.com/";
pub const ENDPOINT_TYPE: &str = "type.googleapis.com/envoy.config.endpoint.v3.ClusterLoadAssignment";
pub const CLUSTER_TYPE: &str = "type.googleapis.com/envoy.config.cluster.v3.Cluster";
pub const ROUTE_TYPE: &str = "type.googleapis.com/envoy.config.route.v3.RouteConfiguration";
pub const SCOPED_ROUTE_TYPE: &str = "type.googleapis.com/envoy.config.route.v3.ScopedRouteConfiguration";
pub const VIRTUAL_HOST_TYPE: &str = "type.googleapis.com/envoy.config.route.v3.VirtualHost";
pub const LISTENER_TYPE: &str = "type.googleapis.com/envoy.config.listener.v3.Listener";
pub const SECRET_TYPE: &str = "type.googleapis.com/envoy.extensions.transport_sockets.tls.v3.Secret";
pub const EXTENSION_CONFIG_TYPE: &str = "type.googleapis.com/envoy.config.core.v3.TypedExtensionConfig";
pub const RUNTIME_TYPE: &str = "type.googleapis.com/envoy.service.runtime.v3.Runtime";
pub const THRIFT_ROUTE_TYPE: &str = "type.googleapis.com/envoy.extensions.filters.network.thrift_proxy.v3.RouteConfiguration";
pub const RATE_LIMIT_CONFIG_TYPE: &str = "type.googleapis.com/ratelimit.config.ratelimit.v3.RateLimitConfig";
pub const ANY_TYPE: &str = "";
pub const HCM_TYPE: &str = "type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager";

// Extensions

pub const GRPC_WEB_TYPE: &str = "type.googleapis.com/envoy.extensions.filters.http.grpc_web.v3.GrpcWeb";

// GetHTTPConnectionManager creates a HttpConnectionManager
// from filter. Returns nil if the filter doesn't have a valid
// HttpConnectionManager configuration.
// pub fn  get_http_connection_manager(filter: listener::Filter) *hcm.HttpConnectionManager {
// 	if typedConfig := filter.GetTypedConfig(); typedConfig != nil {
// 		config := &hcm.HttpConnectionManager{}
// 		if err := anypb.UnmarshalTo(typedConfig, config, proto.UnmarshalOptions{}); err == nil {
// 			return config
// 		}
// 	}

// 	return nil
// }