use tonic::{Request, Response, Status, Streaming};

use crate::{
    resource::ROUTE_TYPE,
    server::service::{Service, StreamResponse},
    cache::{
        Cache, DeltaRequest as DeltaDiscoveryRequest, DeltaResponse as DeltaDiscoveryResponse,
        Request as DiscoveryRequest, Response as DiscoveryResponse,
    },
    envoy::service::route::v3::route_discovery_service_server::RouteDiscoveryService,
};

#[tonic::async_trait]
impl<C: Cache> RouteDiscoveryService for Service<C> {
    type StreamRoutesStream = StreamResponse<DiscoveryResponse>;
    type DeltaRoutesStream = StreamResponse<DeltaDiscoveryResponse>;

    async fn stream_routes(
        &self,
        req: Request<Streaming<DiscoveryRequest>>,
    ) -> Result<Response<Self::StreamRoutesStream>, Status> {
        self.stream(req, ROUTE_TYPE)
    }

    async fn delta_routes(
        &self,
        req: Request<Streaming<DeltaDiscoveryRequest>>,
    ) -> Result<Response<Self::DeltaRoutesStream>, Status> {
        self.delta_stream(req, ROUTE_TYPE)
    }

    async fn fetch_routes(
        &self,
        req: Request<DiscoveryRequest>,
    ) -> Result<Response<DiscoveryResponse>, Status> {
        // self.fetch(req.get_ref(), ROUTE_TYPE).await
        todo!()
    }
}
