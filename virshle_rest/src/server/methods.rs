use axum::response::IntoResponse;
use http_body_util::BodyExt;
use std::vec::Vec;

use std::collections::HashMap;
use uuid::Uuid;

// Node
use crate::server::Server;
use virshle_core::node::{Node, NodeInfo};

pub use pipelight_exec::{Finder, Status};

use crate::commons::{CreateManyVmArgs, CreateVmArgs, GetManyVmArgs, GetVmArgs};
use crate::commons::{
    NodeDefaultMethods, RestDefaultMethods, TemplateDefaultMethods, VmDefaultMethods,
};

// Hypervisor
use virshle_core::{
    config::{Config, UserData, VmTemplate, VmTemplateTable},
    hypervisor::{
        vm::{Vm, VmInfo, VmTable},
        vmm::types::{VmInfoResponse, VmState},
    },
};

// Connections and Http
use virshle_network::http::Rest;

// Error handling
use miette::{Diagnostic, Result};
use tokio::task::JoinError;
use tracing::{error, info, warn};
use virshle_error::{LibError, VirshleError};

impl Server {
    pub fn methods() -> Result<Methods, VirshleError> {
        if let Some(node) = Config::get()?.node {
            let node: Node = node.into();
            return Ok(Methods { node });
        } else {
            let err = LibError::builder()
                .msg("No node running.")
                .help("Create a node first.")
                .build();
            return Err(err.into());
        }
    }
}
impl Methods {
    pub fn node(&self) -> NodeMethods {
        NodeMethods
    }
    pub fn template(&self) -> TemplateMethods {
        TemplateMethods
    }
    pub fn vm(&self) -> VmMethods {
        VmMethods
    }
}
#[derive(Default, Clone)]
struct Methods {
    /// This node alias.
    node: Node,
}

#[derive(Default, Clone)]
struct NodeMethods;
#[derive(Default, Clone)]
struct TemplateMethods;
#[derive(Default, Clone)]
struct VmMethods;

impl NodeDefaultMethods for NodeMethods {
    async fn ping(&self) -> Result<(), VirshleError> {
        Ok(())
    }
    async fn get_info(&self) -> Result<NodeInfo, VirshleError> {
        let res = NodeInfo::get().await?;
        let mut list = HashMap::new();
        // list.insert(self.node, v);
        Ok(res)
    }
}

impl TemplateDefaultMethods for TemplateMethods {
    async fn reclaim(&self, args: CreateVmArgs) -> Result<bool, VirshleError> {
        if let Some(name) = &args.template_name {
            let vm_template = VmTemplate::get_by_name(name)?;
            let can = Node::can_create_vm(&vm_template).await.is_ok();
            Ok(can)
        } else {
            Ok(false)
        }
    }
    async fn get_many(&self) -> Result<HashMap<Node, Vec<VmTemplate>>, VirshleError> {
        let config = Config::get()?;
        if let Some(template) = config.template {
            if let Some(vm_templates) = template.vm {
                let mut list = HashMap::new();
                list.insert(config.nodes(), vm_templates);
                return Ok(list);
            }
        }
        Err(LibError::builder()
            .msg("No template on node.")
            .help("")
            .build()
            .into())
    }
    async fn get_info_many(&self) -> Result<HashMap<Node, Vec<VmTemplateTable>>, VirshleError> {
        let vm_templates = VmTemplate::get_all()?;
        let mut info = vec![];
        for e in vm_templates {
            let table = VmTemplateTable::from(&e)?;
            info.push(table)
        }
        Ok(info)
    }
}

impl VmDefaultMethods for VmMethods {
    async fn start(
        &self,
        args: GetVmArgs,
        user_data: Option<UserData>,
    ) -> Result<Vm, VirshleError> {
        let mut vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        vm.start(user_data.clone(), None).await?;
        Ok(vm)
    }
    async fn start_many(
        &self,
        args: GetManyVmArgs,
        user_data: Option<UserData>,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError> {
        let vms = Vm::database()
            .await?
            .many()
            .maybe_account_uuid(args.account_uuid)
            .maybe_vm_state(args.vm_state)
            .get()
            .await?;

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
        let response = Self::vm_bulk_results_to_response(vms, results)?;
        Self::log_response_op("start", &response)?;
        Ok(response)
    }
    async fn create(
        &self,
        args: CreateVmArgs,
        user_data: Option<UserData>,
    ) -> Result<Vm, VirshleError> {
        let config = Config::get()?;

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
    async fn create_many(
        &self,
        args: CreateManyVmArgs,
        user_data: Option<UserData>,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError> {
        let config = Config::get()?;
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
            let response = Self::vm_bulk_results_to_response(vms, results)?;
            Self::log_response_op("create", &response)?;
            Ok(response)
        } else {
            Err(LibError::builder()
                .msg("Couldn't create Vm")
                .help("No valid template provided")
                .build()
                .into())
        }
    }
    async fn delete(&self, args: GetVmArgs) -> Result<Vm, VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        vm.delete().await?;
        Ok(vm)
    }
    async fn delete_many(
        &self,
        args: GetManyVmArgs,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError> {
        let vms = Vm::database()
            .await?
            .many()
            .maybe_account_uuid(args.account_uuid)
            .maybe_vm_state(args.vm_state)
            .get()
            .await?;

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
        let response = Self::vm_bulk_results_to_response(vms, results)?;
        Self::log_response_op("delete", &response)?;
        Ok(response)
    }

    async fn shutdown(&self, args: GetVmArgs) -> Result<Vm, VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        vm.shutdown().await.ok();
        Ok(vm)
    }
    async fn shutdown_many(
        &self,
        args: GetManyVmArgs,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError> {
        let vms = Vm::database()
            .await?
            .many()
            .maybe_account_uuid(args.account_uuid)
            .maybe_vm_state(args.vm_state)
            .get()
            .await?;
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
        let response = Self::vm_bulk_results_to_response(vms, results)?;
        Self::log_response_op("shutdown", &response)?;
        Ok(response)
    }

    async fn get(&self, args: GetVmArgs) -> Result<Vm, VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        Ok(vm)
    }

    async fn get_many(&self, args: GetManyVmArgs) -> Result<Vec<Vm>, VirshleError> {
        let vms = Vm::database()
            .await?
            .many()
            .maybe_account_uuid(args.account_uuid)
            .maybe_vm_state(args.vm_state)
            .get()
            .await?;
        Ok(vms)
    }
    async fn get_info(&self, args: GetVmArgs) -> Result<VmTable, VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        let table = VmTable::from(&vm).await?;
        Ok(table)
    }
    async fn get_info_many(&self, args: GetManyVmArgs) -> Result<Vec<VmTable>, VirshleError> {
        let vms = Vm::database()
            .await?
            .many()
            .maybe_account_uuid(args.account_uuid)
            .maybe_vm_state(args.vm_state)
            .get()
            .await?;
        let table: Vec<VmTable> = VmTable::from_vec(&vms).await?;
        Ok(table)
    }
}
impl VmMethods {
    /*
     * TODO:
     * It should forward vm tty to user tty or ssh session!
     */
    pub async fn _start_attach(
        args: GetVmArgs,
        user_data: Option<UserData>,
    ) -> Result<Vm, VirshleError> {
        let mut vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        vm.start(user_data.clone(), Some(true)).await?;
        Ok(vm)
    }

    /// Attach a virtual machine on a node.
    pub async fn _attach(args: GetVmArgs, node_name: Option<String>) -> Result<(), VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;

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
    /// Get detailed information about a VM,
    /// from the underlying cloud-hypervisor process.
    pub async fn get_ch_info(&self, args: GetVmArgs) -> Result<VmInfoResponse, VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        let info = vm.vmm().api().info().await?;
        Ok(info.into())
    }
    pub async fn get_raw_ch_info(&self, args: GetVmArgs) -> Result<String, VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        let info = vm.vmm().api()._info().await?;
        Ok(info.into())
    }

    pub async fn ping_ch(&self, args: GetVmArgs) -> Result<(), VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        vm.vmm().api().ping().await
    }
    pub async fn get_vsock_path(&self, args: GetVmArgs) -> Result<String, VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
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

// impl IntoResponse for VmInfoResponse {
//     fn into_response(self) -> axum::response::Response {
//         let json = serde_json::to_string(&self).unwrap();
//         json.into_response()
//     }
// }
// impl IntoResponse for Vm {
//     fn into_response(self) -> axum::response::Response {
//         let json = serde_json::to_string(&self).unwrap();
//         json.into_response()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn test_bulk_result_to_response() -> Result<()> {
        Ok(())
    }
}
