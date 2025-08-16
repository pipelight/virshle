pub mod client;
pub mod method;
pub mod server;

use crate::cloud_hypervisor::VmState;
use serde::{Deserialize, Serialize};
pub use server::NodeRestServer;
use uuid::Uuid;

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
}
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CreateManyVmArgs {
    pub ntimes: Option<u8>,
    pub template_name: Option<String>,
}
