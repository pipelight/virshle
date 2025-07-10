use super::{Vm, VmConfigPlus, VmNet};
use crate::cloud_hypervisor::{Disk, DiskTemplate};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Error Handling
use log::{debug, error, info, trace, warn};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};
/*
* A partial Vm definition, with optional disk, network...
* All those usually mandatory fields will be handled by virshle with
* autoconfigured default.
*/
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct VmTemplate {
    pub name: String,
    pub vcpu: u64,
    pub vram: u64,
    pub uuid: Option<Uuid>,
    pub disk: Option<Vec<DiskTemplate>>,
    pub net: Option<Vec<VmNet>>,
    pub config: Option<VmConfigPlus>,
}
