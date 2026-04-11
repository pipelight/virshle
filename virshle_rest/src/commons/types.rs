use virshle_core::hypervisor::{vm::UserData, vmm::types::VmState};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use pipelight_exec::Status;

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
    pub attach: Option<bool>,
}
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct StartManyVmArgs {
    pub vm_state: Option<VmState>,
    pub account_uuid: Option<Uuid>,
    pub user_data: Option<UserData>,
}
