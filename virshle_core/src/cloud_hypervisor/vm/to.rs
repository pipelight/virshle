use super::{Vm, VmNet};
use std::path::PathBuf;

// Cloud Hypervisor
use uuid::Uuid;
use vmm::api::VmInfoResponse;
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
        let mut config = VmConfig {
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
        if let Some(networks) = &self.net {
            let mut net: Vec<NetConfig> = vec![];
            for def in networks {
                net.push(VmNet::to_vmm_config(def)?);
            }
            config.net = Some(net);
        }
        Ok(config)
    }
}
impl VmNet {
    /*
     * Generate cloud-hypervisor configuration
     */
    pub fn to_vmm_config(&self) -> Result<NetConfig, VirshleError> {
        match self {
            VmNet::Tap(e) => {
                let config = NetConfig {
                    tap: Some(e.name.clone()),
                    ip: default_netconfig_ip(),
                    mask: default_netconfig_mask(),
                    mac: default_netconfig_mac(),
                    host_mac: None,
                    mtu: None,
                    iommu: Default::default(),
                    num_queues: default_netconfig_num_queues(),
                    queue_size: default_netconfig_queue_size(),
                    vhost_user: Default::default(),
                    vhost_socket: None,
                    vhost_mode: Default::default(),
                    id: Default::default(),
                    fds: None,
                    rate_limiter_config: Default::default(),
                    pci_segment: Default::default(),
                    offload_tso: Default::default(),
                    offload_ufo: Default::default(),
                    offload_csum: Default::default(),
                };
                Ok(config)
            }
            VmNet::Bridge(x) => {
                let config = NetConfig {
                    tap: None,
                    ip: default_netconfig_ip(),
                    mask: default_netconfig_mask(),
                    mac: default_netconfig_mac(),
                    host_mac: None,
                    mtu: None,
                    iommu: Default::default(),
                    num_queues: default_netconfig_num_queues(),
                    queue_size: default_netconfig_queue_size(),
                    vhost_user: Default::default(),
                    vhost_socket: None,
                    vhost_mode: Default::default(),
                    id: Default::default(),
                    fds: None,
                    rate_limiter_config: Default::default(),
                    pci_segment: Default::default(),
                    offload_tso: Default::default(),
                    offload_ufo: Default::default(),
                    offload_csum: Default::default(),
                };
                Ok(config)
            }
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn make_vm_from_template() -> Result<()> {
        let toml = "
            vcpu = 1
            vram = 2

            [config]
            autostart = true
        ";

        let item = Vm::from_toml(&toml)?.to_vmm_config()?;
        println!("{:#?}", item);
        Ok(())
    }
    #[test]
    fn make_vm_from_definition_with_ids() -> Result<()> {
        let toml = r#"
            name = "default_xs"
            uuid = "b30458d1-7c7f-4d06-acc2-159e43892e87"

            vcpu = 1
            vram = 2

            [[net]]
            [net.tap]
            name = "macvtap0"

            [[net]]
            [net.bridge]
            name = "virshlebr0"

            "#;
        let item = Vm::from_toml(&toml)?.to_vmm_config()?;
        println!("{:#?}", item);
        Ok(())
    }
}
