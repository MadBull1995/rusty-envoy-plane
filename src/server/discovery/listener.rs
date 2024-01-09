use crate::cache::{Cache, DeltaResponse as DeltaDiscoveryResponse, Response as DiscoveryResponse, Request as DiscoveryRequest, DeltaRequest as DeltaDiscoveryRequest};
use crate::envoy::service::listener::v3::listener_discovery_service_server::ListenerDiscoveryService;
use crate::server::service::{Service, StreamResponse};
use crate::resource::LISTENER_TYPE;
use tonic::{Request, Response, Status, Streaming};

#[tonic::async_trait]
impl<C: Cache> ListenerDiscoveryService for Service<C> {
    type StreamListenersStream = StreamResponse<DiscoveryResponse>;
    type DeltaListenersStream = StreamResponse<DeltaDiscoveryResponse>;

    async fn stream_listeners(
        &self,
        req: Request<Streaming<DiscoveryRequest>>,
    ) -> Result<Response<Self::StreamListenersStream>, Status> {
        self.stream(req, LISTENER_TYPE)
    }

    async fn delta_listeners(
        &self,
        req: Request<Streaming<DeltaDiscoveryRequest>>,
    ) -> Result<Response<Self::DeltaListenersStream>, Status> {
        self.delta_stream(req, LISTENER_TYPE)
    }

    async fn fetch_listeners(
        &self,
        req: Request<DiscoveryRequest>,
    ) -> Result<Response<DiscoveryResponse>, Status> {
        // self.fetch(req.get_ref(), LISTENER_TYPE).await
        todo!()
    }
}
