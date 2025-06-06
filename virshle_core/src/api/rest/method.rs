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

use crate::display::vm::VmTable;
// Hypervisor
use crate::cli::{CreateArgs, VmArgs};
use crate::cloud_hypervisor::{vmm_types::VmInfoResponse, UserData, Vm, VmState, VmTemplate};
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
    pub async fn get_all(Json(params): Json<VmArgs>) -> Result<Json<Vec<VmTable>>, VirshleError> {
        Ok(Json(_get_all(&params).await?))
    }
    pub async fn _get_all(params: &VmArgs) -> Result<Vec<VmTable>, VirshleError> {
        let vm_res = if let Some(state) = &params.state {
            let state = VmState::from_str(&state).unwrap();
            VmTable::from_vec(&Vm::get_by_state(&state).await?).await
        } else {
            VmTable::from_vec(&Vm::get_all().await?).await
        };
        match vm_res {
            Ok(v) => Ok(v),
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
    pub async fn start(
        Json((vm_args, user_data)): Json<(VmArgs, Option<UserData>)>,
    ) -> Result<Json<Vec<Vm>>, VirshleError> {
        Ok(Json(_start(&vm_args, user_data).await?))
    }
    pub async fn _start(
        args: &VmArgs,
        user_data: Option<UserData>,
    ) -> Result<Vec<Vm>, VirshleError> {
        let mut vms = Vm::get_by_args(args).await?;
        for vm in &mut vms {
            vm.start(user_data.clone()).await?;
        }
        Ok(vms)
    }
    pub async fn _start_attach(
        args: &VmArgs,
        user_data: Option<UserData>,
    ) -> Result<Vec<Vm>, VirshleError> {
        let mut vms = Vm::get_by_args(args).await?;
        for vm in &mut vms {
            vm.attach()?.start(user_data.clone()).await?;
        }
        Ok(vms)
    }
    /*
     * Delete a vm and return it.
     */
    pub async fn delete(Json(params): Json<VmArgs>) -> Result<Json<Vec<Vm>>, VirshleError> {
        Ok(Json(_delete(params).await?))
    }
    pub async fn _delete(params: VmArgs) -> Result<Vec<Vm>, VirshleError> {
        let mut vms = Vm::get_by_args(&params).await?;
        for vm in &mut vms {
            vm.delete().await?;
        }
        Ok(vms)
    }
    /*
     * Shutdown a vm and return it.
     */
    pub async fn shutdown(Json(params): Json<VmArgs>) -> Result<Json<Vec<Vm>>, VirshleError> {
        Ok(Json(_shutdown(params).await?))
    }
    pub async fn _shutdown(params: VmArgs) -> Result<Vec<Vm>, VirshleError> {
        let mut vms = Vm::get_by_args(&params).await?;
        for vm in &mut vms {
            vm.shutdown().await?;
        }
        Ok(vms)
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
