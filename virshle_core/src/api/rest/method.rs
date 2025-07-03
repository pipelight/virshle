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
use crate::cloud_hypervisor::{
    vmm_types::VmInfoResponse, UserData, Vm, VmInfo, VmState, VmTemplate,
};
use crate::config::VirshleConfig;

// Error handling
use log::{error, warn};
use miette::{Diagnostic, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub mod node {
    use super::*;
    pub async fn ping() -> Result<(), VirshleError> {
        Ok(())
    }
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
    use crate::cloud_hypervisor::VmConfigPlus;

    /// A strutc to query a VM from a node.
    #[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct GetVmArgs {
        pub id: Option<u64>,
        pub uuid: Option<Uuid>,
        pub name: Option<String>,
    }
    #[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub struct GetManyVmArgs {
        pub vm_state: Option<VmState>,
        pub account_uuid: Option<Uuid>,
    }
    /// Return every VM on node.
    /// Can be filtered by state and/or user account.
    pub async fn get_all(Json(args): Json<GetManyVmArgs>) -> Result<Json<Vec<Vm>>, VirshleError> {
        Ok(Json(_get_all(args).await?))
    }
    pub async fn _get_all(args: GetManyVmArgs) -> Result<Vec<Vm>, VirshleError> {
        Vm::get_many_by_args(args).await
    }

    /// Create a VM on node.
    pub async fn create(
        Json((create_args, vm_config_plus)): Json<(CreateArgs, Option<VmConfigPlus>)>,
    ) -> Result<Json<Vm>, VirshleError> {
        Ok(Json(_create(&create_args, vm_config_plus).await?))
    }
    pub async fn _create(
        create_args: &CreateArgs,
        vm_config_plus: Option<VmConfigPlus>,
    ) -> Result<Vm, VirshleError> {
        let config = VirshleConfig::get()?;

        if let Some(name) = &create_args.template {
            let template = config.get_template(&name)?;
            let mut vm = Vm::from(&template);
            vm.config = vm_config_plus;
            vm.create().await?;
            Ok(vm)
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
    pub async fn start_many(
        Json((args, user_data)): Json<(GetManyVmArgs, Option<UserData>)>,
    ) -> Result<Json<Vec<Vm>>, VirshleError> {
        Ok(Json(_start_many(args, user_data).await?))
    }
    pub async fn _start_many(
        args: GetManyVmArgs,
        user_data: Option<UserData>,
    ) -> Result<Vec<Vm>, VirshleError> {
        let mut vms = Vm::get_many_by_args(args).await?;
        for vm in &mut vms {
            vm.start(user_data.clone(), None).await?;
        }
        Ok(vms)
    }

    /*
     * Start a vm and return it.
     */
    pub async fn start(
        Json((args, user_data)): Json<(GetVmArgs, Option<UserData>)>,
    ) -> Result<Json<Vm>, VirshleError> {
        Ok(Json(_start(args, user_data).await?))
    }
    pub async fn _start(args: GetVmArgs, user_data: Option<UserData>) -> Result<Vm, VirshleError> {
        let mut vm = Vm::get_by_args(&args).await?;
        vm.start(user_data.clone(), None).await?;
        Ok(vm)
    }
    /*
     * TODO:
     * It should forward vm tty to user tty or ssh session!
     */
    pub async fn _start_attach(
        args: &GetVmArgs,
        user_data: Option<UserData>,
    ) -> Result<Vm, VirshleError> {
        let mut vm = Vm::get_by_args(args).await?;
        vm.start(user_data.clone(), Some(true)).await?;
        Ok(vm)
    }

    /// Delete a vm and return it.
    pub async fn delete_many(
        Json(args): Json<GetManyVmArgs>,
    ) -> Result<Json<Vec<Vm>>, VirshleError> {
        Ok(Json(_delete_many(args).await?))
    }
    pub async fn _delete_many(args: GetManyVmArgs) -> Result<Vec<Vm>, VirshleError> {
        let mut vms = Vm::get_many_by_args(args).await?;
        for vm in &mut vms {
            vm.delete().await?;
        }
        Ok(vms)
    }
    // Delete a vm and return it.
    pub async fn delete(Json(args): Json<GetVmArgs>) -> Result<Json<Vm>, VirshleError> {
        Ok(Json(_delete(args).await?))
    }
    pub async fn _delete(args: GetVmArgs) -> Result<Vm, VirshleError> {
        let vm = Vm::get_by_args(&args).await?;
        vm.delete().await?;
        Ok(vm)
    }

    /// Shutdown a vm and return the VM strutct.
    pub async fn shutdown_many(
        Json(args): Json<GetManyVmArgs>,
    ) -> Result<Json<Vec<Vm>>, VirshleError> {
        Ok(Json(_shutdown_many(args).await?))
    }
    pub async fn _shutdown_many(args: GetManyVmArgs) -> Result<Vec<Vm>, VirshleError> {
        let mut vms = Vm::get_many_by_args(args).await?;
        for vm in &mut vms {
            vm.shutdown().await?;
        }
        Ok(vms)
    }
    /// Shutdown a vm and return the VM strutct.
    pub async fn shutdown(Json(args): Json<GetVmArgs>) -> Result<Json<Vm>, VirshleError> {
        Ok(Json(_shutdown(args).await?))
    }
    pub async fn _shutdown(args: GetVmArgs) -> Result<Vm, VirshleError> {
        let vm = Vm::get_by_args(&args).await?;
        vm.shutdown().await?;
        Ok(vm)
    }

    /// Get summarized information about a VM.
    pub async fn get_info(Json(args): Json<GetVmArgs>) -> Result<Json<VmInfo>, VirshleError> {
        Ok(Json(_get_info(&args).await?))
    }
    pub async fn _get_info(args: &GetVmArgs) -> Result<VmInfo, VirshleError> {
        if let Some(id) = args.id {
            let vm = Vm::get_by_id(&id).await?;
            let info = vm.get_info().await?;
            Ok(info)
        } else if let Some(name) = &args.name {
            let vm = Vm::get_by_name(&name).await?;
            let info = vm.get_info().await?;
            Ok(info)
        } else if let Some(uuid) = args.uuid {
            let vm = Vm::get_by_uuid(&uuid).await?;
            let info = vm.get_info().await?;
            Ok(info)
        } else {
            let message = format!("Couldn't find the vm");
            let help = format!("Are you sure the vm exists on this node?");
            Err(LibError::builder().msg(&message).help(&help).build().into())
        }
    }

    /// Get detailed information about a VM,
    /// from the underlying cloud-hypervisor process.
    pub async fn get_ch_info(Json(params): Json<VmArgs>) -> Result<VmInfoResponse, VirshleError> {
        if let Some(id) = params.id {
            let vm = Vm::get_by_id(&id).await?;
            let info = vm.get_ch_info().await?;
            Ok(info.into())
        } else if let Some(name) = params.name {
            let vm = Vm::get_by_name(&name).await?;
            let info = vm.get_ch_info().await?;
            Ok(info)
        } else if let Some(uuid) = params.uuid {
            let vm = Vm::get_by_uuid(&uuid).await?;
            let info = vm.get_ch_info().await?;
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
