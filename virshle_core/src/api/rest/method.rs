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
use crate::config::{Node, NodeInfo};

use crate::display::{VmTable, VmTemplateTable};
// Hypervisor
use crate::cli::{CreateArgs, VmArgs};
use crate::cloud_hypervisor::{
    vmm_types::VmInfoResponse, UserData, Vm, VmInfo, VmState, VmTemplate,
};
use crate::config::VirshleConfig;

// Connections and Http
use crate::connection::{Connection, ConnectionHandle, ConnectionState};
use crate::http_request::{Rest, RestClient};

// Error handling
use log::{error, info, warn};
use miette::{Diagnostic, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub mod node {
    use super::*;

    /// Return info on specified node.
    pub async fn get_info() -> Result<Json<NodeInfo>, VirshleError> {
        Ok(Json(_get_info().await?))
    }
    pub async fn _get_info() -> Result<NodeInfo, VirshleError> {
        let res = NodeInfo::get().await?;
        Ok(res)
    }
    pub async fn ping() -> Result<(), VirshleError> {
        Ok(())
    }
}

pub mod template {
    use super::*;
    use crate::{api::CreateVmArgs, display::vm_template, Node};

    /// Ask node if a vm can be created from template.
    pub async fn reclaim(Json(args): Json<CreateVmArgs>) -> Result<Json<bool>, VirshleError> {
        Ok(Json(_reclaim(args).await?))
    }
    pub async fn _reclaim(args: CreateVmArgs) -> Result<bool, VirshleError> {
        if let Some(name) = &args.template_name {
            let vm_template = VmTemplate::get_by_name(name)?;
            let can = Node::can_create_vm(&vm_template).await.is_ok();
            Ok(can)
        } else {
            Ok(false)
        }
    }

    /// Get summarized information about a VM.
    pub async fn get_info_many() -> Result<Json<Vec<VmTemplateTable>>, VirshleError> {
        Ok(Json(_get_info_many().await?))
    }
    pub async fn _get_info_many() -> Result<Vec<VmTemplateTable>, VirshleError> {
        let vm_templates = VmTemplate::get_all()?;
        let mut info = vec![];
        for e in vm_templates {
            let table = VmTemplateTable::from(&e)?;
            info.push(table)
        }
        Ok(info)
    }

    /// Return all template name.
    pub async fn get_all() -> Result<Json<Vec<VmTemplate>>, VirshleError> {
        Ok(Json(_get_all().await?))
    }
    pub async fn _get_all() -> Result<Vec<VmTemplate>, VirshleError> {
        let config = VirshleConfig::get()?;
        if let Some(template) = config.template {
            if let Some(vm_templates) = template.vm {
                return Ok(vm_templates);
            }
        }
        Err(LibError::builder()
            .msg("No template on node.")
            .help("")
            .build()
            .into())
    }
}

pub mod vm {
    use pipelight_exec::Finder;

    use super::*;
    use crate::api::{CreateVmArgs, GetManyVmArgs, GetVmArgs};
    use crate::cloud_hypervisor::VmConfigPlus;

    /// Return every VM on node.
    /// Can be filtered by state and/or user account.
    pub async fn get_all(Json(args): Json<GetManyVmArgs>) -> Result<Json<Vec<Vm>>, VirshleError> {
        Ok(Json(_get_all(args).await?))
    }
    pub async fn _get_all(args: GetManyVmArgs) -> Result<Vec<Vm>, VirshleError> {
        Vm::get_many_by_args(&args).await
    }

    /// Create a VM on node.
    pub async fn create(
        Json((args, user_data)): Json<(CreateVmArgs, Option<UserData>)>,
    ) -> Result<Json<Vm>, VirshleError> {
        Ok(Json(_create(args, user_data).await?))
    }
    pub async fn _create(
        args: CreateVmArgs,
        user_data: Option<UserData>,
    ) -> Result<Vm, VirshleError> {
        let config = VirshleConfig::get()?;

        if let Some(name) = &args.template_name {
            let template = config.get_template(&name)?;

            // Safeguard before creating.
            Node::can_create_vm(&template).await?;

            let mut vm = Vm::from(&template)?;
            vm.create(user_data).await?;
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
        let mut vms = Vm::get_many_by_args(&args).await?;
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
        args: GetVmArgs,
        user_data: Option<UserData>,
    ) -> Result<Vm, VirshleError> {
        let mut vm = Vm::get_by_args(&args).await?;
        vm.start(user_data.clone(), Some(true)).await?;
        Ok(vm)
    }

    /// Attach a virtual machine on a node.
    pub async fn _attach(args: GetVmArgs, node_name: Option<String>) -> Result<(), VirshleError> {
        let vm = Vm::get_by_args(&args).await?;

        let finder = Finder::new()
            .seed("cloud-hypervisor")
            .seed(&vm.uuid.to_string())
            .search_no_parents()?;

        #[cfg(debug_assertions)]
        if let Some(matches) = finder.matches {
            if let Some(_match) = matches.first() {
                if let Some(pid) = _match.pid {}
            }
        }

        Ok(())
    }

    /// Delete a vm and return it.
    pub async fn delete_many(
        Json(args): Json<GetManyVmArgs>,
    ) -> Result<Json<Vec<Vm>>, VirshleError> {
        Ok(Json(_delete_many(args).await?))
    }
    pub async fn _delete_many(args: GetManyVmArgs) -> Result<Vec<Vm>, VirshleError> {
        let mut vms = Vm::get_many_by_args(&args).await?;
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
        let mut vms = Vm::get_many_by_args(&args).await?;
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
        vm.shutdown().await.ok();
        Ok(vm)
    }

    /// Get summarized information about a VM.
    pub async fn get_info_many(
        Json(args): Json<GetManyVmArgs>,
    ) -> Result<Json<Vec<VmTable>>, VirshleError> {
        Ok(Json(_get_info_many(args).await?))
    }
    pub async fn _get_info_many(args: GetManyVmArgs) -> Result<Vec<VmTable>, VirshleError> {
        let vms = Vm::get_many_by_args(&args).await?;
        let table: Vec<VmTable> = VmTable::from_vec(&vms).await?;
        Ok(table)
    }

    /// Get detailed information about a VM,
    /// from the underlying cloud-hypervisor process.
    pub async fn get_ch_info(Json(args): Json<GetVmArgs>) -> Result<VmInfoResponse, VirshleError> {
        let vm = Vm::get_by_args(&args).await?;
        let info = vm.get_ch_info().await?;
        Ok(info.into())
    }
    pub async fn ping_ch(Json(args): Json<GetVmArgs>) -> Result<(), VirshleError> {
        let vm = Vm::get_by_args(&args).await?;
        vm.ping_ch().await
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
