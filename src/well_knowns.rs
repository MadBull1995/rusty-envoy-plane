macro_rules! prefix {
    ($type:literal) => {
        concat!("type.googleapis.com/", $type)
    };
}

// HTTP filter names
pub const BUFFER: &str = prefix!("envoy.filters.http.buffer");
pub const CORS: &str = prefix!("envoy.filters.http.cors");
// ... other HTTP filter names ...

// Network filter names
pub const CLIENT_SSL_AUTH: &str = prefix!("envoy.filters.network.client_ssl_auth");
pub const ECHO: &str = prefix!("envoy.filters.network.echo");
pub const HTTP_CONNECTION_MANAGER: &str = prefix!("envoy.filters.network.http_connection_manager");

// Listener filter names
pub const ORIGINAL_DESTINATION: &str = prefix!("envoy.filters.listener.original_dst");
pub const PROXY_PROTOCOL: &str = prefix!("envoy.filters.listener.proxy_protocol");
// ... other listener filter names ...

// Tracing provider names
pub const LIGHTSTEP: &str = prefix!("envoy.tracers.lightstep");
pub const ZIPKIN: &str = prefix!("envoy.tracers.zipkin");
// ... other tracing provider names ...

// Stats sink names
pub const STATSD: &str = prefix!("envoy.stat_sinks.statsd");
pub const DOG_STATSD: &str = prefix!("envoy.stat_sinks.dog_statsd");
// ... other stats sink names ...

// Access log sink names
pub const FILE_ACCESS_LOG: &str = prefix!("envoy.access_loggers.file");
pub const HTTP_GRPC_ACCESS_LOG: &str = prefix!("envoy.access_loggers.http_grpc");
// ... other access log sink names ...

// Transport socket names
pub const TRANSPORT_SOCKET_ALTS: &str = prefix!("envoy.transport_sockets.alts");
pub const TRANSPORT_SOCKET_TAP: &str = prefix!("envoy.transport_sockets.tap");
// ... other transport socket names ...