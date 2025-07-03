use axum::{
    extract::{Extension, Path, Query},
    http::Request,
    middleware::map_response,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};

use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use uuid::Uuid;

use tokio::net::{UnixListener, UnixStream};

use crate::api::{method, NodeServer};
use crate::config::NodeInfo;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use sysinfo::System;

// Error handling
use miette::{Diagnostic, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub struct NodeRestServer;
impl NodeRestServer {
    pub async fn make_router() -> Result<Router, VirshleError> {
        // Cloud-hypervisor direct calls.
        let api_v1_ch = Router::new()
            // Vm
            .route(
                "/vm.info",
                get(async move |params| method::vm::get_ch_info(params).await),
            )
            .route(
                "/vmm.ping",
                get(async move |params| method::vm::get_ch_info(params).await),
            );

        // Virshle API
        let api_v1 = Router::new()
            // Node
            // Check for the REST API availability
            .route("/node/ping", get(async || method::node::ping().await))
            .route("/node/info", get(async || method::node::get_info().await))
            // Template
            .route(
                "/template/all",
                get(async || method::template::get_all().await),
            )
            // Vm
            .route(
                "/vm/all",
                post(async move |params| method::vm::get_all(params).await),
            )
            .route(
                "/vm/create",
                put(async move |params| method::vm::create(params).await),
            )
            .route(
                "/vm/info",
                put(async move |params| method::vm::get_info(params).await),
            )
            .route(
                "/vm/start",
                put(async move |params| method::vm::start(params).await),
            )
            .route(
                "/vm/shutdown",
                put(async move |params| method::vm::shutdown(params).await),
            )
            .route(
                "/vm/delete",
                put(async move |params| method::vm::delete(params).await),
            );

        let app = Router::new()
            .nest("/api/v1", api_v1)
            .nest("/api/v1/ch", api_v1_ch)
            .layer(map_response(Self::set_header))
            .layer(TraceLayer::new_for_http());

        Ok(app)
    }

    async fn set_header<B>(mut response: Response<B>) -> Response<B> {
        response
            .headers_mut()
            .insert("server", "Virshle API".parse().unwrap());
        response
    }

    /*
     * Run REST api only.
     */
    pub async fn run() -> Result<(), VirshleError> {
        let app = NodeRestServer::make_router().await?;

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
        NodeRestServer::run().await?;
        Ok(())
    }
}
