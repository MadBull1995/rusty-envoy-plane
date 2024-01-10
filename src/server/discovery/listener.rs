use tonic::{Request, Response, Status, Streaming};

use crate::{
    resource::LISTENER_TYPE,
    server::service::{Service, StreamResponse},
    cache::{
        Cache, DeltaRequest as DeltaDiscoveryRequest, DeltaResponse as DeltaDiscoveryResponse,
        Request as DiscoveryRequest, Response as DiscoveryResponse,
    },
    envoy::service::listener::v3::listener_discovery_service_server::ListenerDiscoveryService,
};


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
