use axum::{
    body::Body,
    extract::{Extension, Path, Query},
    http::Request,
    response::{IntoResponse, Response},
    Json, Router,
};
use http_body_util::BodyExt;
use hyper::{body::Bytes, StatusCode};
use std::str::FromStr;
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
use log::{error, warn};
use miette::{Diagnostic, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub mod node {
    use super::*;
    pub async fn get_info() -> Result<String, VirshleError> {
        let host = NodeInfo::get().await?;
        let info = serde_json::to_string(&host)?;
        Ok(info)
    }
}

pub mod template {
    use super::*;
    pub async fn get_all() -> Result<String, VirshleError> {
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
}

pub mod vm {
    use super::*;
    pub async fn get_all(Json(params): Json<VmArgs>) -> Result<Json<Vec<Vm>>, VirshleError> {
        let vm_res = if let Some(state) = params.state {
            let state = VmState::from_str(&state).unwrap();
            Vm::get_by_state(state).await
        } else {
            Vm::get_all().await
        };
        match vm_res {
            Ok(v) => Ok(Json(v)),
            Err(e) => {
                error!("{}", e);
                Err(e)
            }
        }
    }
    /*
     * Get node info (cpu, ram...)
     */
    // pub async fn create_vm(Json(params): Json<CreateArgs>) -> Result<Vec<Vm>, VirshleError> {
    pub async fn create(Json(params): Json<CreateArgs>) -> Result<String, VirshleError> {
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
     * Start a vm and return it.
     */
    pub async fn start(Json(params): Json<VmArgs>) -> Result<Json<Vec<Vm>>, VirshleError> {
        if let Some(id) = params.id {
            let mut vm = Vm::get_by_id(&id).await?;
            vm.start().await?;
            let vms = vec![vm];
            Ok(Json(vms))
        } else if let Some(name) = params.name {
            let mut vm = Vm::get_by_name(&name).await?;
            vm.start().await?;
            let vms = vec![vm];
            Ok(Json(vms))
        } else if let Some(uuid) = params.uuid {
            let mut vm = Vm::get_by_uuid(&uuid).await?;
            vm.start().await?;
            let vms = vec![vm];
            Ok(Json(vms))
        } else if let Some(state) = params.state {
            let state = VmState::from_str(&state).unwrap();
            let mut vms = Vm::get_by_state(state).await?;
            for vm in &mut vms {
                vm.start().await?;
            }
            Ok(Json(vms))
        } else {
            let message = format!("Couldn't find vm.");
            let help = format!("Are you sure the vm exists on this node?");
            Err(LibError::builder().msg(&message).help(&help).build().into())
        }
    }
    /*
     * Delete a vm and return it.
     */
    pub async fn delete(Json(params): Json<VmArgs>) -> Result<Json<Vec<Vm>>, VirshleError> {
        if let Some(id) = params.id {
            let vm = Vm::get_by_id(&id).await?;
            vm.delete().await?;
            let vms = vec![vm];
            Ok(Json(vms))
        } else if let Some(name) = params.name {
            let vm = Vm::get_by_name(&name).await?;
            vm.delete().await?;
            let vms = vec![vm];
            Ok(Json(vms))
        } else if let Some(uuid) = params.uuid {
            let vm = Vm::get_by_uuid(&uuid).await?;
            vm.delete().await?;
            let vms = vec![vm];
            Ok(Json(vms))
        } else if let Some(state) = params.state {
            let state = VmState::from_str(&state).unwrap();
            let mut vms = Vm::get_by_state(state).await?;
            for vm in &mut vms {
                vm.delete().await?;
            }
            Ok(Json(vms))
        } else {
            let message = format!("Couldn't find vm.");
            let help = format!("Are you sure the vm exists on this node?");
            Err(LibError::builder().msg(&message).help(&help).build().into())
        }
    }
    /*
     * Shutdown a vm and return it.
     */
    pub async fn shutdown(Json(params): Json<VmArgs>) -> Result<Json<Vec<Vm>>, VirshleError> {
        if let Some(id) = params.id {
            let vm = Vm::get_by_id(&id).await?;
            vm.shutdown().await?;
            let vms = vec![vm];
            Ok(Json(vms))
        } else if let Some(name) = params.name {
            let vm = Vm::get_by_name(&name).await?;
            vm.shutdown().await?;
            let vms = vec![vm];
            Ok(Json(vms))
        } else if let Some(uuid) = params.uuid {
            let vm = Vm::get_by_uuid(&uuid).await?;
            vm.shutdown().await?;
            let vms = vec![vm];
            Ok(Json(vms))
        } else if let Some(state) = params.state {
            let state = VmState::from_str(&state).unwrap();
            let mut vms = Vm::get_by_state(state).await?;
            for vm in &mut vms {
                vm.shutdown().await?;
            }
            Ok(Json(vms))
        } else {
            let message = format!("Couldn't find vm.");
            let help = format!("Are you sure the vm exists on this node?");
            Err(LibError::builder().msg(&message).help(&help).build().into())
        }
    }
    pub async fn get_info(Json(params): Json<VmArgs>) -> Result<VmInfoResponse, VirshleError> {
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
