use super::Vm;
use std::path::PathBuf;

// Cloud Hypervisor
use uuid::Uuid;
use vmm::api::VmInfoResponse;
use vmm::{
    vm::VmState,
    vm_config::{
        // defaults
        default_console,
        default_serial,

        CpusConfig,
        DiskConfig,
        MemoryConfig,
        NetConfig,
        PayloadConfig,
        RngConfig,
        VmConfig,
    },
};

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};
use virshle_error::{LibError, VirshleError};

impl Vm {
    /*
     * Generate cloud-hypervisor configuration
     */
    pub fn to_vmm_config(&self) -> Result<VmConfig, VirshleError> {
        let kernel = "/run/cloud-hypervisor/hypervisor-fw";
        let config = VmConfig {
            cpus: CpusConfig {
                boot_vcpus: self.vcpu as u8,
                max_vcpus: self.vcpu as u8,
                ..Default::default()
            },
            memory: MemoryConfig {
                size: self.vram * u64::pow(1024, 2),
                ..Default::default()
            },

            disks: None,
            net: None,

            // Unused params
            payload: Some(PayloadConfig {
                kernel: Some(PathBuf::from(kernel)),
                firmware: None,
                cmdline: None,
                initramfs: None,
            }),
            rate_limit_groups: Default::default(),
            rng: Default::default(),
            balloon: Default::default(),
            fs: Default::default(),
            pmem: Default::default(),
            serial: default_serial(),
            console: default_console(),
            debug_console: Default::default(),
            devices: Default::default(),
            user_devices: Default::default(),
            vdpa: Default::default(),
            vsock: Default::default(),
            pvpanic: Default::default(),
            iommu: Default::default(),
            sgx_epc: Default::default(),
            numa: Default::default(),
            watchdog: Default::default(),
            pci_segments: Default::default(),
            platform: Default::default(),
            tpm: Default::default(),
            preserved_fds: Default::default(),
            landlock_rules: Default::default(),
            landlock_enable: Default::default(),
        };
        Ok(config)
    }
}
