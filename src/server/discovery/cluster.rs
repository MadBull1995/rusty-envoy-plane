use tonic::{Request, Response, Status, Streaming};

use crate::{
    resource::CLUSTER_TYPE,
    server::service::{Service, StreamResponse},
    cache::{
        Cache, DeltaRequest as DeltaDiscoveryRequest, DeltaResponse as DeltaDiscoveryResponse,
        Request as DiscoveryRequest, Response as DiscoveryResponse,
    },
    envoy::service::cluster::v3::cluster_discovery_service_server::ClusterDiscoveryService,
};
#[tonic::async_trait]
impl<C: Cache> ClusterDiscoveryService for Service<C> {
    type StreamClustersStream = StreamResponse<DiscoveryResponse>;
    type DeltaClustersStream = StreamResponse<DeltaDiscoveryResponse>;

    async fn stream_clusters(
        &self,
        req: Request<Streaming<DiscoveryRequest>>,
    ) -> Result<Response<Self::StreamClustersStream>, Status> {
        self.stream(req, CLUSTER_TYPE)
    }

    async fn delta_clusters(
        &self,
        req: Request<Streaming<DeltaDiscoveryRequest>>,
    ) -> Result<Response<Self::DeltaClustersStream>, Status> {
        self.delta_stream(req, CLUSTER_TYPE)
    }

    async fn fetch_clusters(
        &self,
        req: Request<DiscoveryRequest>,
    ) -> Result<Response<DiscoveryResponse>, Status> {
        // self.fetch(req.get_ref(), CLUSTER_TYPE).await
        todo!()
    }
}
