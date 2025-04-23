use axum::{
    extract::connect_info::{self, ConnectInfo},
    http::Request,
    routing::get,
    Router,
};
use hyper::body::Incoming;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server,
};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::{unix::UCred, UnixListener, UnixStream};
use tower::Service;

use std::path::PathBuf;

// Hypervisor
use crate::cloud_hypervisor::Vm;

// Error handling
use log::info;
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub struct Api;

impl Api {
    async fn get_all_vms() -> Result<String, VirshleError> {
        let vms = serde_json::to_string(&Vm::get_all().await?)?;
        Ok(vms)
    }
    pub async fn run() -> Result<(), VirshleError> {
        // build our application with a single route
        let app = Router::new()
            .route("/vm/list", get(Self::get_all_vms().await.unwrap()))
            .route(
                "/vm/create/{template}",
                get(|| async {
                    // serde_json::to_string(&Vm::set().await.unwrap());
                }),
            )
            .route(
                "/node",
                get(|| async {
                    serde_json::to_string(&Vm::get_all().await.unwrap());
                }),
            )
            .route("/", get(|| async { "Hello, World!" }));

        let path = "/var/lib/virshle/virshle.sock";
        let path = PathBuf::from(path);

        // Ensure clean socket
        let _ = tokio::fs::remove_file(&path).await;
        tokio::fs::create_dir_all(path.parent().unwrap())
            .await
            .unwrap();

        let listener = UnixListener::bind("/var/lib/virshle/virshle.sock")?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_run() -> Result<()> {
        Api::run().await?;
        Ok(())
    }
}
