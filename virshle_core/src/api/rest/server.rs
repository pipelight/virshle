use axum::{
    extract::{Extension, Path, Query},
    http::Request,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};

use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use uuid::Uuid;

use tokio::net::{UnixListener, UnixStream};

use crate::api::{NodeMethod, NodeServer};
use crate::config::NodeInfo;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use sysinfo::System;

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub struct RestServer;
impl RestServer {
    pub async fn make_router() -> Result<Router, VirshleError> {
        // build our application with a single route
        let app = Router::new()
            // Template
            .route(
                "/template/list",
                get(NodeMethod::get_all_template().await.unwrap()),
            )
            // Vm
            .route("/vm/list", get(NodeMethod::get_all_vm().await.unwrap()))
            .route(
                "/vm/create",
                put(async move |params| {
                    NodeMethod::create_vm(params).await.unwrap();
                }),
            )
            .route(
                "/vm/info",
                put(async move |params| {
                    NodeMethod::get_vm_info(params).await.unwrap();
                }),
            )
            .route(
                "/vm/start",
                put(async move |params| {
                    NodeMethod::start_vm(params).await.unwrap();
                }),
            )
            // .route(
            //     "/vm/stop",
            //     put(async move |params| {
            //         Self::start_vm(params).await.unwrap();
            //     }),
            // )
            // Node
            .route(
                "/node/info",
                get(NodeMethod::get_node_info().await.unwrap()),
            )
            .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

        Ok(app)
    }

    /*
     * Run REST api only.
     */
    pub async fn run() -> Result<(), VirshleError> {
        let app = RestServer::make_router().await?;

        let listener = NodeServer::make_socket().await?;
        axum::serve(listener, app).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_run() -> Result<()> {
        RestServer::run().await?;
        Ok(())
    }
}
