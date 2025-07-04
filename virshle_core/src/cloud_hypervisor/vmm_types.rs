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
use super::{vm::NetType, Disk, Vm};
use crate::network::{ip::fd, utils};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::path::PathBuf;
use std::str::FromStr;

// Error handling
use log::info;
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

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
            readonly: e.readonly.unwrap_or(Default::default()),
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
#[skip_serializing_none]
#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct NetConfig {
    num_queues: Option<u64>,
    pub mac: Option<String>,
    pub host_mac: Option<String>,

    // tap
    pub tap: Option<String>,
    pub fd: Option<Vec<i32>>,

    // dpdk
    pub vhost_mode: Option<VhostMode>,
    pub vhost_user: Option<bool>,
    pub vhost_socket: Option<String>,

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

impl FromStr for VmState {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let res = match s {
            "not_created" => VmState::NotCreated,
            "created" => VmState::Created,
            "running" => VmState::Running,
            "shutdown" => VmState::Shutdown,
            "paused" => VmState::Paused,
            "breakpoing" => VmState::BreakPoint,
            _ => VmState::Running,
        };
        Ok(res)
    }
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

        // Attach/Detach from standard output.
        config.serial = Some(ConsoleConfig {
            mode: ConsoleOutputMode::Tty,
        });

        config.console = match e.is_attach().unwrap() {
            true => Some(ConsoleConfig {
                mode: ConsoleOutputMode::Tty,
            }),
            false => Some(ConsoleConfig {
                mode: ConsoleOutputMode::Off,
            }),
        };

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
                let port_name = format!("vm-{}-{}", e.name, net.name);

                match &net._type {
                    NetType::Vhost(_) => {
                        net_configs.push(NetConfig {
                            mac: Some(utils::uuid_to_mac(&e.uuid).to_string()),
                            // dpdk specific
                            vhost_user: Some(true),
                            vhost_mode: Some(VhostMode::Server),
                            vhost_socket: e.get_net_socket(&net).ok(),

                            // multiqueue support
                            // num_queues: Some(e.vcpu * 2),
                            ..Default::default()
                        });
                    }
                    NetType::Tap(_) => {
                        // external Tap via name
                        let tap_name = utils::unix_name(&port_name);
                        net_configs.push(NetConfig {
                            mac: Some(utils::uuid_to_mac(&e.uuid).to_string()),
                            //tap
                            tap: Some(tap_name),

                            // multiqueue support
                            // num_queues: Some(e.vcpu * 2),
                            ..Default::default()
                        });

                        // external Tap via file descriptor
                        // let fd = fd::get_fd(&port_name).unwrap();
                        // net_configs.push(NetConfig {
                        //     fd: Some(vec![fd]),
                        //     ..Default::default()
                        // });
                    }
                    NetType::MacVTap(_) => {
                        // external Tap via name
                        let tap_name = utils::unix_name(&port_name);
                        net_configs.push(NetConfig {
                            //tap
                            tap: Some(tap_name),

                            // multiqueue support
                            num_queues: Some(2),
                            ..Default::default()
                        });
                    }
                }
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
        name = "main"
        [net.type.vhost]

        "#;

        let vm_template = VmTemplate::from_toml(&toml)?;
        println!("{:#?}", vm_template);

        let vm = Vm::from(&vm_template);
        println!("{:#?}", vm);

        let vmm_config = VmConfig::from(&vm);

        println!("{:#?}", vmm_config);
        Ok(())
    }

    #[test]
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
        name = "main"
        [net.type.vhost]
        "#;

        let vm = Vm::from_toml(&toml)?;
        let vmm_config = VmConfig::from(&vm);

        // println!("{:#?}", vmm_config);
        Ok(())
    }
}
