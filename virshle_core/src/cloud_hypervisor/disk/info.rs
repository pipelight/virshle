use crate::cloud_hypervisor::{Disk, DiskTemplate};
use serde::{Deserialize, Serialize};

use crate::display::utils::{display_some_bool, display_some_bytes};
use tabled::Tabled;

// Error Handling
use log::{log_enabled, Level};
use miette::{IntoDiagnostic, Result};
use virshle_error::VirshleError;

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Tabled)]
pub struct DiskInfo {
    pub name: String,
    pub path: String,
    #[tabled(display = "display_some_bytes")]
    pub size: Option<u64>,
    #[tabled(display = "display_some_bool")]
    pub readonly: Option<bool>,
}

impl DiskInfo {
    pub fn from(e: &Disk) -> Result<Self, VirshleError> {
        let info = DiskInfo {
            name: e.name.clone(),
            path: e.path.clone(),
            size: e.get_size().ok(),
            readonly: e.readonly,
        };
        Ok(info)
    }
    pub fn from_template(e: &DiskTemplate) -> Result<Self, VirshleError> {
        let info = DiskInfo {
            name: e.name.clone(),
            path: e.path.clone(),
            size: e.get_size().ok(),
            readonly: e.readonly,
        };
        Ok(info)
    }
}
