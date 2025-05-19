pub mod grpc;
pub mod methods;
pub mod rest;

// Globals
use crate::config::MANAGED_DIR;

use axum_tonic::RestGrpcService;

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub struct Server;

impl Server {
    /*
     * Run REST api and gRPC on same socket.
     */
    pub async fn run() -> Result<(), VirshleError> {
        let rest_router = Self::make_rest_router().await?;
        let grpc_router = Self::make_grpc_router().await?;
        let service = RestGrpcService::new(rest_router, grpc_router).into_make_service();

        Ok(())
    }
}

impl Server {
    /*
     * Return the virshle daemon default socket path.
     */
    pub fn get_socket() -> Result<String, VirshleError> {
        let path = format!("{MANAGED_DIR}/virshle.sock");
        Ok(path)
    }
    pub fn get_host() -> Result<(), VirshleError> {
        Ok(())
    }
}
