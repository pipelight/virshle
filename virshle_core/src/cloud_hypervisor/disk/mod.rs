use std::fs;
use std::path::PathBuf;

// Cloud Hypervisor
use uuid::Uuid;
use vmm::api::VmInfoResponse;
use vmm::vm_config::DiskConfig as ChDiskConfig;
use vmm::{
    vm::VmState,
    vm_config::{
        // defaults vm
        default_console,
        default_serial,
    },
    vm_config::{
        // defaults net
        default_netconfig_ip,
        default_netconfig_mac,
        default_netconfig_mask,
        default_netconfig_num_queues,
        default_netconfig_queue_size,
        default_netconfig_tap,
        default_netconfig_true,
    },
    vm_config::{
        CpusConfig, DiskConfig, MemoryConfig, NetConfig, PayloadConfig, RngConfig, VmConfig,
    },
};

use serde::{Deserialize, Serialize};

// Error Handling
use miette::{IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};
use virshle_error::{LibError, VirshleError};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct DiskTemplate {
    pub path: String,
    pub readonly: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Disk {
    pub path: String,
    pub readonly: Option<bool>,
}
impl From<&DiskTemplate> for Disk {
    fn from(e: &DiskTemplate) -> Self {
        Self {
            path: e.path.to_owned(),
            readonly: e.readonly,
        }
    }
}
impl Disk {
    pub fn to_vmm_config(&self) -> Result<DiskConfig, VirshleError> {
        let config = DiskConfig {
            path: Some(PathBuf::from(&self.path)),
            readonly: Default::default(),
            direct: Default::default(),
            iommu: Default::default(),
            num_queues: Default::default(),
            queue_size: Default::default(),
            vhost_user: Default::default(),
            vhost_socket: None,
            id: Default::default(),
            rate_limit_group: Default::default(),
            rate_limiter_config: Default::default(),
            disable_io_uring: Default::default(),
            disable_aio: Default::default(),
            pci_segment: Default::default(),
            serial: Default::default(),
            queue_affinity: Default::default(),
        };
        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Error Handling
    use miette::{IntoDiagnostic, Result};

    #[test]
    fn make_handled_disk() -> Result<()> {
        Ok(())
    }
}
