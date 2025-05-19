use tarpc::{
    client, context, context::Context;
    server::{self, incoming::Incoming, Channel},
};
use axum::Router;
use tonic::{transport::Server as GrpcServer, Request, Response, Status};
use axum_tonic::NestTonic;

// Hypervisor
use crate::cli::VmArgs;

use crate::config::NodeInfo;

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

use super::Server;

#[derive(Default,Clone)]
struct GrpcService;

#[tonic_rpc::tonic_rpc(json)]
trait Vm {
    async fn node_info() -> NodeInfo;
}

#[tonic::async_trait]
impl Vm for GrpcService {
    async fn node_info(self,context: Context) -> NodeInfo {
        NodeInfo::get().await.unwrap()
    }
}

impl Server {
    pub async fn run_grpc() -> Result<(), VirshleError> {
        Ok(())
    }
    pub async fn make_grpc_router() -> Result<Router, VirshleError> {
        let grpc_service = GrpcService::default();
        let app = Router::new().nest_tonic(GrpcServer::new(grpc_service));
        Ok(app)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_run() -> Result<()> {
        Server::run().await?;
        Ok(())
    }
}
