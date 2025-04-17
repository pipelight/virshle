/*
* This file is a loose mapping to cloud-hypervisor(ch) types.
*
* Types in here are only use to convert
* simple virshle types into cloud-hypervisor types
*
* We can then generate a json,
* to be sent to cloud-hypervisor http api,
* in just a few lines.
*/
use super::{Disk, Vm};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Cpu
#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ConsoleConfig {
    pub mode: ConsoleOutputMode,
}
#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum ConsoleOutputMode {
    #[default]
    Off,
    Pty,
    Tty,
    File,
    Socket,
    Null,
}
// Cpu
#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct CpusConfig {
    pub boot_vcpus: u64,
    pub max_vcpus: u64,
    // Removed ch unused default
    #[serde(flatten)]
    other: serde_json::Value,
}

// Ram
#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct MemoryConfig {
    pub size: u64,
    pub shared: bool,
    pub hugepages: bool,
    #[serde(default)]
    pub hugepage_size: Option<u64>,
    // Removed ch unused default
    #[serde(flatten)]
    other: serde_json::Value,
}

// Disk
#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct DiskConfig {
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub readonly: bool,
    // Removed ch unused default
    #[serde(flatten)]
    other: serde_json::Value,
}
impl From<&Disk> for DiskConfig {
    fn from(e: &Disk) -> Self {
        DiskConfig {
            path: Some(PathBuf::from(&e.path)),
            readonly: Default::default(),
            ..Default::default()
        }
    }
}
// Disk efi bootloader
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PayloadConfig {
    pub kernel: Option<String>,
    pub cmdline: Option<String>,
}

// Network
#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct NetConfig {
    pub vhost_user: bool,
    pub vhost_socket: Option<String>,
    #[serde(default)]
    pub vhost_mode: VhostMode,
    num_queues: Option<u64>,
    // Removed ch unused default
    #[serde(flatten)]
    other: serde_json::Value,
}
#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum VhostMode {
    #[default]
    Client,
    Server,
}

// Http API
/*
* Following types are use to convert http api responses
* to simpler structs for virshle to display.
*/

#[derive(Default, Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum VmState {
    #[default]
    NotCreated,
    Created,
    Running,
    Shutdown,
    Paused,
    BreakPoint,
}
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct VmInfoResponse {
    pub config: VmConfig,
    pub state: VmState,
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct VmConfig {
    pub cpus: CpusConfig,
    pub memory: MemoryConfig,
    pub disks: Option<Vec<DiskConfig>>,
    pub net: Option<Vec<NetConfig>>,

    // Hardcoded into virshle for now.
    pub payload: Option<PayloadConfig>,
    pub console: Option<ConsoleConfig>,
    pub serial: Option<ConsoleConfig>,

    #[serde(flatten)]
    other: serde_json::Value,
}

impl From<&Vm> for VmConfig {
    fn from(e: &Vm) -> Self {
        // Todo(): make those values dynamic
        let kernel = "/run/cloud-hypervisor/hypervisor-fw";

        let mut config = VmConfig {
            cpus: CpusConfig {
                boot_vcpus: e.vcpu,
                max_vcpus: e.vcpu,
                ..Default::default()
            },
            memory: MemoryConfig {
                size: e.vram * u64::pow(1024, 3),
                shared: true,
                hugepages: true,
                ..Default::default()
            },
            disks: None,
            net: None,
            ..Default::default()
        };

        // Add bootloader
        let payload = PayloadConfig {
            kernel: Some("/run/cloud-hypervisor/hypervisor-fw".to_owned()),
            cmdline: Some(
                "earlyprintk=ttyS0 console=ttyS0 console=hvc0 root=/dev/vda1 rw".to_owned(),
            ),
        };
        config.payload = Some(payload);
        config.console = Some(ConsoleConfig {
            mode: ConsoleOutputMode::Off,
        });
        config.serial = Some(ConsoleConfig {
            mode: ConsoleOutputMode::Tty,
        });

        // Add disks
        let mut disk: Vec<DiskConfig> = vec![];
        for def in &e.disk {
            disk.push(DiskConfig::from(def));
        }
        config.disks = Some(disk);

        // Add networks
        if let Some(nets) = &e.net {
            let mut net_configs: Vec<NetConfig> = vec![];
            for net in nets {
                net_configs.push(NetConfig {
                    vhost_user: true,
                    num_queues: Some(e.vcpu * 2),
                    vhost_socket: e.get_net_socket(&net).ok(),
                    vhost_mode: VhostMode::Server,
                    ..Default::default()
                });
            }
            config.net = Some(net_configs);
        }
        config
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cloud_hypervisor::VmTemplate;
    use miette::{IntoDiagnostic, Result};
    use std::path::PathBuf;

    #[test]
    fn make_vm_from_template() -> Result<()> {
        let toml = r#"
        name = "xs"
        vcpu = 1
        vram = 2

        [[disk]]
        name = "os"
        path = "~/Iso/nixos.efi.img"
        size = "50G"

        [[net]]
        [net.vhost]
        "#;

        let vm_template = VmTemplate::from_toml(&toml)?;
        println!("{:#?}", vm_template);

        let vm = Vm::from(&vm_template);
        println!("{:#?}", vm);

        let vmm_config = VmConfig::from(&vm);

        println!("{:#?}", vmm_config);
        Ok(())
    }

    // #[test]
    fn make_vm_from_definition_with_ids() -> Result<()> {
        let toml = r#"
        name = "test_xs"
        uuid = "b30458d1-7c7f-4d06-acc2-159e43892e87"
        vcpu = 1
        vram = 2

        [[disk]]
        name = "os"
        path = "~/Iso/nixos.efi.img"
        size = "50G"

        [[net]]
        [net.vhost]
        "#;

        let vm = Vm::from_toml(&toml)?;
        let vmm_config = VmConfig::from(&vm);

        // println!("{:#?}", vmm_config);
        Ok(())
    }
}
