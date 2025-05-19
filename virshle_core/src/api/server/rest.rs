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

use crate::config::NodeInfo;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use sysinfo::System;

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

use super::Server;

impl Server {
    pub async fn make_rest_router() -> Result<Router, VirshleError> {
        // build our application with a single route
        let app = Router::new()
            // Template
            .route(
                "/template/list",
                get(Self::get_all_template().await.unwrap()),
            )
            // Vm
            .route("/vm/list", get(Self::get_all_vm().await.unwrap()))
            .route(
                "/vm/create",
                put(async move |params| {
                    Self::create_vm(params).await.unwrap();
                }),
            )
            .route(
                "/vm/info",
                put(async move |params| {
                    Self::get_vm_info(params).await.unwrap();
                }),
            )
            .route(
                "/vm/start",
                put(async move |params| {
                    Self::start_vm(params).await.unwrap();
                }),
            )
            // .route(
            //     "/vm/stop",
            //     put(async move |params| {
            //         Self::start_vm(params).await.unwrap();
            //     }),
            // )
            // Node
            .route("/node/info", get(Self::get_node_info().await.unwrap()))
            .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));
        Ok(app)
    }

    /*
     * Run REST api only.
     */
    pub async fn run_rest() -> Result<(), VirshleError> {
        let app = Self::make_rest_router().await?;
        let listener = Self::make_socket().await?;
        axum::serve(listener, app).await?;
        Ok(())
    }

    /*
     * Create a unix socket with custom permissions.
     */
    pub async fn make_socket() -> Result<UnixListener, VirshleError> {
        let socket = Self::get_socket()?;
        let path = PathBuf::from(socket);

        // Remove old socket.
        let _ = tokio::fs::remove_file(&path).await;
        tokio::fs::create_dir_all(path.parent().unwrap())
            .await
            .unwrap();

        // Create new socket.
        let listener = UnixListener::bind(&path)?;

        // Set permissions
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o774);
        fs::set_permissions(&path, perms)?;

        Ok(listener)
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
