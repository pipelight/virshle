use virshle_core::{
    config::{VmTemplate, VmTemplateTable},
    hypervisor::{
        vm::{UserData, Vm, VmInfo, VmTable},
        vmm::types::{VmInfoResponse, VmState},
    },
    node::{NodeInfo, Peer},
};
use virshle_network::connection::{Connection, ConnectionHandle, ConnectionState};
use virshle_network::http::{Rest, RestClient};

pub use pipelight_exec::Status;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Error handling
use miette::Result;
use tracing::warn;
use virshle_error::VirshleError;

pub trait RestDefaultMethods {
    fn node(&self) -> impl NodeDefaultMethods;
    fn template(&self) -> impl TemplateDefaultMethods;
    fn vm(&self) -> impl VmDefaultMethods;
}

pub trait NodeDefaultMethods {
    async fn ping(&self) -> Result<(), VirshleError>;
    async fn get_info(&self, alias: Option<String>) -> Result<NodeInfo, VirshleError>;
    // async fn get_info_many(&self) -> Result<HashMap<Node, NodeInfo>, VirshleError>;
}
pub trait NodeManyDefaultMethods {
    async fn ping(&self) -> Result<(), VirshleError>;
    async fn get_info(&self) -> Result<NodeInfo, VirshleError>;
    // async fn get_info_many(&self) -> Result<HashMap<Node, NodeInfo>, VirshleError>;
}
pub trait TemplateDefaultMethods {
    async fn get_many(&self) -> Result<HashMap<Peer, Vec<VmTemplate>>, VirshleError>;
    async fn get_info_many(&self) -> Result<HashMap<Peer, Vec<VmTemplateTable>>, VirshleError>;
    async fn reclaim(&self, args: CreateVmArgs) -> Result<bool, VirshleError>;
}

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

pub async fn alerte_connection_state(rest: &RestClient) -> Result<(), VirshleError> {
    // Logging
    let state = rest.connection.get_state().await?;
    let message = format!("node {:#?} unreachable", node.name);
    match state {
        ConnectionState::SshAuthError => {
            let message = format!("node {:#?} ssh authenticaton rejected", node.name);
            warn!("{}", &message)
        }
        ConnectionState::Unreachable => {
            let message = format!("node {:#?} is unreachable", node.name);
            warn!("{}", &message)
        }
        ConnectionState::Down => {
            let message = format!("node {:#?} host is down", node.name);
            warn!("{}", &message)
        }
        ConnectionState::DaemonDown => {
            let message = format!("node {:#?} daemon is down", node.name);
            warn!("{}", &message)
        }
        ConnectionState::SocketNotFound => {
            let message = format!("node {:#?} no socket found", node.name);
            warn!("{}", &message)
        }
        _ => {}
    };
    Ok(())
}
