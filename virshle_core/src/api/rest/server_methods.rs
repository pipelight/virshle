use axum::{
    body::Body,
    extract::{Extension, Path, Query},
    http::Request,
    response::{IntoResponse, Response},
    Json, Router,
};
use http_body_util::BodyExt;
use hyper::{body::Bytes, StatusCode};
use std::vec::Vec;

use std::collections::HashMap;
use uuid::Uuid;

use serde::{Deserialize, Serialize};
// Node
use crate::config::NodeInfo;

// Hypervisor
use crate::cli::{CreateArgs, VmArgs};
use crate::cloud_hypervisor::{vmm_types::VmInfoResponse, Vm, VmState, VmTemplate};
use crate::config::VirshleConfig;

// Error handling
use miette::{Diagnostic, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub struct NodeMethod;
impl NodeMethod {
    pub async fn get_all_vm() -> Result<Json<Vec<Vm>>, VirshleError> {
        match Vm::get_all().await {
            Ok(v) => Ok(Json(v)),
            Err(e) => Err(e),
        }
    }
    pub async fn get_all_template() -> Result<String, VirshleError> {
        let config = VirshleConfig::get()?;
        if let Some(template) = config.template {
            let templates = serde_json::to_string(&template.vm)?;
            Ok(templates)
        } else {
            return Err(LibError::builder()
                .msg("No template on node.")
                .help("")
                .build()
                .into());
        }
    }
    /*
     * Get node info (cpu, ram...)
     */
    pub async fn get_node_info() -> Result<String, VirshleError> {
        let host = NodeInfo::get().await?;
        let info = serde_json::to_string(&host)?;
        Ok(info)
    }

    // pub async fn create_vm(Json(params): Json<CreateArgs>) -> Result<Vec<Vm>, VirshleError> {
    pub async fn create_vm(Json(params): Json<CreateArgs>) -> Result<String, VirshleError> {
        let config = VirshleConfig::get()?;

        if let Some(name) = params.template {
            let template = config.get_template(&name)?;
            let mut vm = Vm::from(&template);
            vm.create().await?;
            let vms = vec![vm];
            let vms = serde_json::to_string(&vms)?;
            Ok(vms)
        } else {
            Err(LibError::builder()
                .msg("Couldn't create Vm")
                .help("No valid template provided")
                .build()
                .into())
        }
    }

    /*
     * Return a string
     */
    pub async fn start_vm(Json(params): Json<VmArgs>) -> Result<String, VirshleError> {
        if let Some(id) = params.id {
            let mut vm = Vm::get_by_id(&id).await?;
            vm.start().await?;
            let vms = vec![vm];
            let vms = serde_json::to_string(&vms)?;
            Ok(vms)
        } else if let Some(name) = params.name {
            let mut vm = Vm::get_by_name(&name).await?;
            vm.start().await?;
            let vms = vec![vm];
            let vms = serde_json::to_string(&vms)?;
            Ok(vms)
        } else if let Some(uuid) = params.uuid {
            let mut vm = Vm::get_by_uuid(&uuid).await?;
            vm.start().await?;
            let vms = vec![vm];
            let vms = serde_json::to_string(&vms)?;
            Ok(vms)
        } else if let Some(state) = params.state {
            let state: VmState = serde_json::from_str(&state)?;
            let mut vms = Vm::get_by_state(state).await?;
            for vm in &mut vms {
                vm.start().await?;
            }
            let vms = serde_json::to_string(&vms)?;
            Ok(vms)
        } else {
            let message = format!("Couldn't find vm.");
            let help = format!("Are you sure the vm exists on this node?");
            Err(LibError::builder().msg(&message).help(&help).build().into())
        }
    }
    pub async fn get_vm_info(Json(params): Json<VmArgs>) -> Result<VmInfoResponse, VirshleError> {
        // println!("{:#?}", params);
        if let Some(id) = params.id {
            let vm = Vm::get_by_id(&id).await?;
            let info = vm.get_info().await?;
            Ok(info.into())
        } else if let Some(name) = params.name {
            let vm = Vm::get_by_name(&name).await?;
            let info = vm.get_info().await?;
            Ok(info)
        } else if let Some(uuid) = params.uuid {
            let vm = Vm::get_by_uuid(&uuid).await?;
            let info = vm.get_info().await?;
            Ok(info)
        } else {
            let message = format!("Couldn't find vm.");
            let help = format!("Are you sure the vm exists on this node?");
            Err(LibError::builder().msg(&message).help(&help).build().into())
        }
    }
    pub async fn stop_vm(
        Query(params): Query<HashMap<String, String>>,
    ) -> Result<(), VirshleError> {
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
}

/*
* Compatibility with axum
* Transform Ok and Err types that can be serialized to json.
*/
impl NodeMethod {
    pub fn into_response<T>(result: Result<T, VirshleError>) -> impl IntoResponse
    where
        T: std::fmt::Debug + Serialize,
    {
        match result {
            Ok(v) => Response::builder()
                .status(StatusCode::OK)
                .body(Body::new(serde_json::to_string(&v).unwrap()))
                .unwrap(),
            Err(e) => e.into_response(),
        }
    }
}

impl IntoResponse for VmInfoResponse {
    fn into_response(self) -> axum::response::Response {
        let json = serde_json::to_string(&self).unwrap();
        json.into_response()
    }
}
impl IntoResponse for Vm {
    fn into_response(self) -> axum::response::Response {
        let json = serde_json::to_string(&self).unwrap();
        json.into_response()
    }
}
