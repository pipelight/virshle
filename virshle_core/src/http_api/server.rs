use axum::{
    extract::{
        connect_info::{self, ConnectInfo},
        Extension, Path, Query,
    },
    http::Request,
    routing::{get, put},
    Json, Router,
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use uuid::Uuid;

use std::collections::HashMap;
use tokio::net::{UnixListener, UnixStream};

use std::path::PathBuf;

// Globals
use crate::config::MANAGED_DIR;

// Hypervisor
use crate::cloud_hypervisor::{Vm, VmTemplate};
use crate::config::VirshleConfig;

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub struct Server;

impl Server {
    /*
     * Return the virshle daemon default socket path.
     */
    pub fn get_socket() -> Result<String, VirshleError> {
        let path = format!("{MANAGED_DIR}/virshle.sock");
        Ok(path)
    }
    async fn get_all_vm() -> Result<String, VirshleError> {
        let vms = serde_json::to_string(&Vm::get_all().await?)?;
        Ok(vms)
    }
    async fn get_all_template() -> Result<String, VirshleError> {
        let config = VirshleConfig::get()?;
        if let Some(template) = config.template {
            let templates = serde_json::to_string(&template.vm)?;
            Ok(templates)
        } else {
            return Err(LibError::new("No template on node.", "").into());
        }
    }
    async fn create_vm(Path(template_name): Path<String>) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;
        let template = config.get_template(&template_name)?;
        let mut vm = Vm::from(&template);
        vm.create().await?;
        Ok(())
    }
    async fn start_vm(Query(params): Query<HashMap<String, String>>) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;
        if let Some(id) = params.get("id") {
            let mut vm = Vm::get_by_id(&id.parse()?).await?;
            vm.start().await?;
        } else if let Some(name) = params.get("name") {
            let mut vm = Vm::get_by_name(&name).await?;
            vm.start().await?;
        } else if let Some(uuid) = params.get("uuid") {
            let mut vm = Vm::get_by_uuid(&Uuid::parse_str(uuid)?).await?;
            vm.start().await?;
        }
        Ok(())
    }
    async fn stop_vm(Query(params): Query<HashMap<String, String>>) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;
        if let Some(id) = params.get("id") {
            let vm = Vm::get_by_id(&id.parse()?).await?;
            vm.shutdown().await?;
        } else if let Some(name) = params.get("name") {
            let vm = Vm::get_by_name(&name).await?;
            vm.shutdown().await?;
        } else if let Some(uuid) = params.get("uuid") {
            let vm = Vm::get_by_uuid(&Uuid::parse_str(uuid)?).await?;
            vm.shutdown().await?;
        }
        Ok(())
    }
    async fn get_vm_info(
        Query(params): Query<HashMap<String, String>>,
    ) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;
        if let Some(id) = params.get("id") {
            let vm = Vm::get_by_id(&id.parse()?).await?;
            vm.get_info().await?;
        } else if let Some(name) = params.get("name") {
            let vm = Vm::get_by_name(&name).await?;
            vm.get_info().await?;
        } else if let Some(uuid) = params.get("uuid") {
            let vm = Vm::get_by_uuid(&Uuid::parse_str(uuid)?).await?;
            vm.get_info().await?;
        }
        Ok(())
    }

    pub async fn run() -> Result<(), VirshleError> {
        // build our application with a single route
        let app = Router::new()
            .route(
                "/template/list",
                get(Self::get_all_template().await.unwrap()),
            )
            .route("/vm/list", get(Self::get_all_vm().await.unwrap()))
            .route(
                "/vm/create/{template_name}",
                put(async move |path| {
                    Self::create_vm(path).await.unwrap();
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
            .route(
                "/vm/stop",
                put(async move |params| {
                    Self::start_vm(params).await.unwrap();
                }),
            )
            .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

        let socket = Self::get_socket()?;
        let path = PathBuf::from(socket);

        // Remove old socket.
        let _ = tokio::fs::remove_file(&path).await;
        tokio::fs::create_dir_all(path.parent().unwrap())
            .await
            .unwrap();

        // Create new socket.
        let listener = UnixListener::bind(path)?;
        axum::serve(listener, app).await?;

        Ok(())
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
