use owo_colors::OwoColorize;
use virshle_core::{
    config::{VmTemplate, VmTemplateTable},
    hypervisor::{
        vm::{UserData, Vm, VmInfo, VmTable},
        vmm::types::{VmInfoResponse, VmState},
    },
    peer::{NodeInfo, Peer},
};
use virshle_network::connection::{Connection, ConnectionHandle, ConnectionState};
use virshle_network::http::{Rest, RestClient};

pub use pipelight_exec::Status;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Error handling
use miette::Result;
use tokio::task::JoinError;
use tracing::{info, warn};
use virshle_error::VirshleError;

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
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CreateVmArgs {
    pub template_name: Option<String>,
    pub user_data: Option<UserData>,
}
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CreateManyVmArgs {
    pub ntimes: Option<u8>,
    pub template_name: Option<String>,
    pub user_data: Option<UserData>,
}
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct StartVmArgs {
    pub id: Option<u64>,
    pub uuid: Option<Uuid>,
    pub name: Option<String>,
    pub user_data: Option<UserData>,
}
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct StartManyVmArgs {
    pub vm_state: Option<VmState>,
    pub account_uuid: Option<Uuid>,
    pub user_data: Option<UserData>,
}

pub trait VmDefaultMethods {
    async fn get(&self, args: GetVmArgs) -> Result<Vm, VirshleError>;
    async fn get_many(&self, args: GetManyVmArgs) -> Result<HashMap<Peer, Vec<Vm>>, VirshleError>;

    async fn create(
        &self,
        args: CreateVmArgs,
        user_data: Option<UserData>,
    ) -> Result<Vm, VirshleError>;
    async fn create_many(
        &self,
        args: CreateManyVmArgs,
        user_data: Option<UserData>,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError>;

    async fn start(&self, args: GetVmArgs, user_data: Option<UserData>)
        -> Result<Vm, VirshleError>;
    async fn start_many(
        &self,
        args: GetManyVmArgs,
        user_data: Option<UserData>,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError>;

    async fn shutdown(&self, args: GetVmArgs) -> Result<Vm, VirshleError>;
    async fn shutdown_many(
        &self,
        args: GetManyVmArgs,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError>;

    async fn delete(&self, args: GetVmArgs) -> Result<Vm, VirshleError>;
    async fn delete_many(
        &self,
        args: GetManyVmArgs,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError>;

    async fn get_info(&self, args: GetVmArgs) -> Result<VmTable, VirshleError>;
    async fn get_info_many(&self, args: GetManyVmArgs) -> Result<Vec<VmTable>, VirshleError>;
}

pub async fn alerte_connection_state(
    peer: &Peer,
    rest: &mut RestClient,
) -> Result<(), VirshleError> {
    // Logging
    let state = rest.connection.get_state().await?;
    let alias = peer.alias()?;
    match state {
        ConnectionState::SshAuthError => {
            let message = format!("peer {:#?} ssh authenticaton rejected", alias);
            warn!("{}", &message)
        }
        ConnectionState::Unreachable => {
            let message = format!("peer {:#?} is unreachable", alias);
            warn!("{}", &message)
        }
        ConnectionState::Down => {
            let message = format!("peer {:#?} host is down", alias);
            warn!("{}", &message)
        }
        ConnectionState::DaemonDown => {
            let message = format!("peer {:#?} daemon is down", alias);
            warn!("{}", &message)
        }
        ConnectionState::SocketNotFound => {
            let message = format!("peer {:#?} no socket found", alias);
            warn!("{}", &message)
        }
        _ => {}
    };
    Ok(())
}

/// Log response
// pub fn log_response(
//     tag: &str,
//     node: &str,
//     response: &HashMap<Peer, Result<Vec<Vm>, VirshleError>>,
// ) -> Result<(), VirshleError> {
//     let tag = format!("[{tag}]");
//     for (peer, res) in response.iter() {
//         match v {
//             Ok(x) => {
//                 let tag = tag.red();
//                 let vms_name: Vec<String> = v.iter().map(|e| e.name.to_owned()).collect();
//                 let vms_name = vms_name.join(" ");
//                 info!("{tag} failed for vms [{}] on node {node}", vms_name);
//             }
//             Status::Succeeded => {
//                 let tag = tag.green();
//                 let vms_name: Vec<String> = v.iter().map(|e| e.name.to_owned()).collect();
//                 let vms_name = vms_name.join(" ");
//                 info!("{tag} succedded for vms [{}] on node {node}", vms_name);
//             }
//             _ => {}
//         }
//     }
//     Ok(())
// }

/// Convert bulk operations result like start.many
/// into HashMap of successful and failed operations.
#[tracing::instrument]
pub async fn vm_bulk_results_to_hashmap(
    vms: Vec<Vm>,
    results: Vec<Result<Result<Vm, VirshleError>, JoinError>>,
) -> Result<HashMap<Status, Vec<VmTable>>, VirshleError> {
    let mut response: HashMap<Status, Vec<VmTable>> =
        HashMap::from([(Status::Succeeded, vec![]), (Status::Failed, vec![])]);
    for res in results {
        match res? {
            Err(_) => {}
            Ok(vm) => {
                let vm = VmTable::from(&vm).await?;
                response.get_mut(&Status::Succeeded).unwrap().push(vm);
            }
        }
    }
    // Vm not contained in Result::Ok() are by deduction in Err().
    // Can't do a comparison on Vm to Vm because some actions mutates
    // the vm so it will always return a false so we must use the Vm uuid.
    let succeeded_uuid: Vec<Uuid> = response
        .get(&Status::Succeeded)
        .unwrap()
        .iter()
        .map(|e| e.uuid)
        .collect();
    let mut failed: Vec<VmTable> = vec![];
    for vm in vms {
        if !succeeded_uuid.contains(&vm.uuid) {
            let vm = VmTable::from(&vm).await?;
            failed.push(vm)
        }
    }
    response.get_mut(&Status::Failed).unwrap().extend(failed);
    Ok(response)
}

/// Log response
#[tracing::instrument(skip(response), name = "bulk op")]
pub fn log_response_op(tag: &str, response: &HashMap<Status, Vec<Vm>>) -> Result<(), VirshleError> {
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
