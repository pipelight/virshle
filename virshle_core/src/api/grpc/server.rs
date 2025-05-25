use axum::Router;
use tonic::{body::Body, transport::Server, Request, Response, Status};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::api::NodeServer;

// Socket to Stream
use hyper_util::rt::TokioIo;
use std::path::Path;

use tokio::net::UnixStream;
use tokio_stream::wrappers::UnixListenerStream;

// Hypervisor
use crate::cli::VmArgs;
use crate::config::NodeInfo;

// Error handling
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

#[tonic_rpc::tonic_rpc(json)]
trait GetInfo {
    async fn node_info() -> NodeInfo;
}

#[derive(Default, Clone)]
pub struct NodeService;

#[tonic::async_trait]
impl get_info_server::GetInfo for NodeService {
    async fn node_info(&self, request: Request<()>) -> Result<Response<NodeInfo>, Status> {
        let res = NodeInfo::get().await.unwrap();
        Ok(Response::new(res))
    }
}

pub struct NodeGrpcServer;
impl NodeGrpcServer {
    /*
     * Run grpc server on unix socket
     */
    pub async fn run() -> Result<(), VirshleError> {
        let service = get_info_server::GetInfoServer::new(NodeService);

        let uds = NodeServer::make_socket().await?;
        let uds_stream = UnixListenerStream::new(uds);

        Server::builder()
            .add_service(service)
            .serve_with_incoming(uds_stream)
            .await
            .unwrap();

        Ok(())
    }

    pub fn make_router() -> Result<Router, VirshleError> {
        let service = get_info_server::GetInfoServer::new(NodeService);
        let app = Router::new()
            .route("/grpc", axum::routing::any_service(service))
            .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

        Ok(app)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_run_grpc() -> Result<()> {
        NodeGrpcServer::run().await?;
        Ok(())
    }
}
