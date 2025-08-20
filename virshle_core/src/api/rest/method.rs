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

// Global vars
use std::sync::{Arc, Mutex};

use std::collections::HashMap;
use uuid::Uuid;

use serde::{Deserialize, Serialize};
// Node
use crate::config::{Node, NodeInfo};

use crate::display::{VmTable, VmTemplateTable};
// Hypervisor
use crate::cli::{CreateArgs, VmArgs};
use crate::cloud_hypervisor::vm::{
    to_vmm_types::{VmInfoResponse, VmState},
    UserData, Vm, VmInfo, VmTemplate,
};
use crate::config::VirshleConfig;

// Connections and Http
use crate::connection::{Connection, ConnectionHandle, ConnectionState};
use crate::http_request::{Rest, RestClient};

use owo_colors::OwoColorize;
// Error handling
use log::{error, info, warn};
use miette::{Diagnostic, IntoDiagnostic, Result};
use tokio::task::JoinError;
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
    pub use pipelight_exec::{Finder, Status};

    use super::*;
    use crate::api::{CreateManyVmArgs, CreateVmArgs, GetManyVmArgs, GetVmArgs};
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
            vm = vm.create(user_data).await?;
            Ok(vm)
        } else {
            Err(LibError::builder()
                .msg("Couldn't create Vm")
                .help("No valid template provided")
                .build()
                .into())
        }
    }
    /// Create many vms and return them.
    pub async fn create_many(
        Json((args, user_data)): Json<(CreateManyVmArgs, Option<UserData>)>,
    ) -> Result<Json<HashMap<Status, Vec<Vm>>>, VirshleError> {
        Ok(Json(_create_many(args, user_data).await?))
    }
    pub async fn _create_many(
        args: CreateManyVmArgs,
        user_data: Option<UserData>,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError> {
        let config = VirshleConfig::get()?;
        if args.template_name.is_some() && args.ntimes.is_some() {
            let template = config.get_template(&args.template_name.unwrap())?;

            let mut tasks = vec![];
            let mut vms = vec![];
            for i in 0..args.ntimes.unwrap() {
                Node::can_create_vm(&template).await?;
                let mut vm = Vm::from(&template)?;
                tasks.push(tokio::spawn({
                    let user_data = user_data.clone();
                    async move {
                        let mut vm = vm.clone();
                        vm.create(user_data).await
                    }
                }));
            }
            let results: Vec<Result<Result<Vm, VirshleError>, JoinError>> =
                futures::future::join_all(tasks).await;
            let response = vm_bulk_results_to_response(vms, results)?;
            log_response_op("create", &response)?;
            Ok(response)
        } else {
            Err(LibError::builder()
                .msg("Couldn't create Vm")
                .help("No valid template provided")
                .build()
                .into())
        }
    }
    /// Start a vm and return it.
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
    /// Start multiple vm and return them.
    pub async fn start_many(
        Json((args, user_data)): Json<(GetManyVmArgs, Option<UserData>)>,
    ) -> Result<Json<HashMap<Status, Vec<Vm>>>, VirshleError> {
        Ok(Json(_start_many(args, user_data).await?))
    }
    pub async fn _start_many(
        args: GetManyVmArgs,
        user_data: Option<UserData>,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError> {
        let mut vms: Vec<Vm> = Vm::get_many_by_args(&args).await?;

        let mut tasks = vec![];
        for mut vm in vms.clone() {
            tasks.push(tokio::spawn({
                let user_data = user_data.clone();
                async move {
                    let mut vm = vm.clone();
                    vm.start(user_data, None).await
                }
            }));
        }
        let results: Vec<Result<Result<Vm, VirshleError>, JoinError>> =
            futures::future::join_all(tasks).await;
        let response = vm_bulk_results_to_response(vms, results)?;
        log_response_op("start", &response)?;
        Ok(response)
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
    pub async fn delete(Json(args): Json<GetVmArgs>) -> Result<Json<Vm>, VirshleError> {
        Ok(Json(_delete(args).await?))
    }
    pub async fn _delete(args: GetVmArgs) -> Result<Vm, VirshleError> {
        let vm = Vm::get_by_args(&args).await?;
        vm.delete().await?;
        Ok(vm)
    }

    /// Delete a vm and return it.
    pub async fn delete_many(
        Json(args): Json<GetManyVmArgs>,
    ) -> Result<Json<HashMap<Status, Vec<Vm>>>, VirshleError> {
        Ok(Json(_delete_many(args).await?))
    }
    pub async fn _delete_many(
        args: GetManyVmArgs,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError> {
        let vms = Vm::get_many_by_args(&args).await?;

        let mut tasks = vec![];
        for vm in vms.clone() {
            tasks.push(tokio::spawn({
                async move {
                    let vm = vm.clone();
                    vm.delete().await
                }
            }));
        }
        let results: Vec<Result<Result<Vm, VirshleError>, JoinError>> =
            futures::future::join_all(tasks).await;
        let response = vm_bulk_results_to_response(vms, results)?;
        log_response_op("delete", &response)?;
        Ok(response)
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
    /// Shutdown a vm and return the VM strutct.
    pub async fn shutdown_many(
        Json(args): Json<GetManyVmArgs>,
    ) -> Result<Json<HashMap<Status, Vec<Vm>>>, VirshleError> {
        Ok(Json(_shutdown_many(args).await?))
    }
    pub async fn _shutdown_many(
        args: GetManyVmArgs,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError> {
        let vms = Vm::get_many_by_args(&args).await?;
        let mut tasks = vec![];
        for vm in vms.clone() {
            tasks.push(tokio::spawn({
                async move {
                    let vm = vm.clone();
                    vm.shutdown().await
                }
            }));
        }
        let results: Vec<Result<Result<Vm, VirshleError>, JoinError>> =
            futures::future::join_all(tasks).await;
        let response = vm_bulk_results_to_response(vms, results)?;
        log_response_op("shutdown", &response)?;
        Ok(response)
    }

    /// Get a VM definition.
    pub async fn get(Json(args): Json<GetVmArgs>) -> Result<Json<Vm>, VirshleError> {
        Ok(Json(_get(args).await?))
    }
    pub async fn _get(args: GetVmArgs) -> Result<Vm, VirshleError> {
        let vm = Vm::get_by_args(&args).await?;
        Ok(vm)
    }

    /// Get vm info.
    pub async fn get_info(Json(args): Json<GetVmArgs>) -> Result<Json<VmTable>, VirshleError> {
        Ok(Json(_get_info(args).await?))
    }
    pub async fn _get_info(args: GetVmArgs) -> Result<VmTable, VirshleError> {
        let vm = Vm::get_by_args(&args).await?;
        let table = VmTable::from(&vm).await?;
        Ok(table)
    }

    /// Get summarized information about a VMs.
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
        vm.ping_ch().await?;
        let info = vm.get_ch_info().await?;
        Ok(info.into())
    }
    pub async fn get_raw_ch_info(Json(args): Json<GetVmArgs>) -> Result<String, VirshleError> {
        let vm = Vm::get_by_args(&args).await?;
        vm.ping_ch().await?;
        let info = vm.get_raw_ch_info().await?;
        Ok(info.into())
    }

    pub async fn ping_ch(Json(args): Json<GetVmArgs>) -> Result<(), VirshleError> {
        let vm = Vm::get_by_args(&args).await?;
        vm.ping_ch().await
    }
    pub async fn get_vsock_path(Json(args): Json<GetVmArgs>) -> Result<String, VirshleError> {
        let vm = Vm::get_by_args(&args).await?;
        let path = vm.get_vsocket()?;
        Ok(path)
    }
    /// Convert bulk operations result like start.many
    /// into HashMap of successful and failed operations.
    #[tracing::instrument]
    pub fn vm_bulk_results_to_response(
        vms: Vec<Vm>,
        results: Vec<Result<Result<Vm, VirshleError>, JoinError>>,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError> {
        let mut response: HashMap<Status, Vec<Vm>> =
            HashMap::from([(Status::Succeeded, vec![]), (Status::Failed, vec![])]);
        for res in results {
            match res? {
                Err(e) => {}
                Ok(vm) => {
                    response.get_mut(&Status::Succeeded).unwrap().push(vm);
                }
            }
        }

        // Vm not contained in Result::Ok() or by deduction in Err().
        // Can't do a comparison on Vm to Vm because some actions mutates
        // the vm so it will always return a false so we must use the Vm uuid.
        let succeeded_uuid: Vec<Uuid> = response
            .get(&Status::Succeeded)
            .unwrap()
            .iter()
            .map(|e| e.uuid)
            .collect();
        let failed: Vec<Vm> = vms
            .iter()
            .filter(|e| !succeeded_uuid.contains(&e.uuid))
            .map(|e| e.to_owned())
            .collect();

        response.get_mut(&Status::Failed).unwrap().extend(failed);
        Ok(response)
    }
    /// Log response
    #[tracing::instrument(skip(response), name = "bulk op")]
    pub fn log_response_op(
        tag: &str,
        response: &HashMap<Status, Vec<Vm>>,
    ) -> Result<(), VirshleError> {
        let tag = format!("[bulk-op][{tag}]");
        for (k, v) in response.iter() {
            match k {
                Status::Failed => {
                    let tag = tag.red();
                    if !v.is_empty() {
                        let vms_name: Vec<String> = v.iter().map(|e| e.name.to_owned()).collect();
                        let vms_name = vms_name.join(" ");
                        info!("{tag} failed for vms [{}]", vms_name);
                    }
                }
                Status::Succeeded => {
                    let tag = tag.green();
                    if !v.is_empty() {
                        let vms_name: Vec<String> = v.iter().map(|e| e.name.to_owned()).collect();
                        let vms_name = vms_name.join(" ");
                        info!("{tag} succeeded for vms [{}]", vms_name);
                    }
                }
                _ => {}
            }
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::vm::*;
    use super::*;

    // #[test]
    fn test_bulk_result_to_response() -> Result<()> {
        Ok(())
    }
}
