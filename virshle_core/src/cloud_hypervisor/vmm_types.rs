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
use crate::{
    config::VirshleConfig,
    network::{dhcp::DhcpType, ip::fd, utils},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::path::PathBuf;
use std::str::FromStr;

use std::future::Future;
use std::net::IpAddr;

// Error handling
use log::info;
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

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
    pub vsock: Option<VsockConfig>,

    // Memory
    pub balloon: Option<BalloonConfig>,

    #[serde(flatten)]
    other: serde_json::Value,
}

// Cpu
#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ConsoleConfig {
    pub mode: ConsoleOutputMode,
    pub socket: Option<String>,
}

// Disk efi bootloader
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PayloadConfig {
    pub kernel: Option<String>,
    pub cmdline: Option<String>,
}

// Console
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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct VsockConfig {
    pub cid: u32,
    pub socket: String,
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
    pub hotplug_size: Option<u64>,
    // Removed ch unused default
    #[serde(flatten)]
    other: serde_json::Value,
}
#[derive(Default, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BalloonConfig {
    pub size: Option<u64>,
    pub deflate_on_oom: Option<bool>,
    pub free_page_reporting: Option<bool>,
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

// Network
#[skip_serializing_none]
#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct NetConfig {
    num_queues: Option<u64>,
    pub mac: Option<String>,
    pub host_mac: Option<String>,

    pub ip: Option<IpAddr>,
    pub mask: Option<IpAddr>,

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
    // cloud-hypervisor
    #[default]
    NotCreated,
    Created,
    Running,
    Shutdown,
    Paused,
    BreakPoint,
}

impl FromStr for VmState {
    type Err = VirshleError;
    fn from_str(s: &str) -> Result<Self, VirshleError> {
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

// impl From<&Vm> for VmConfig {
// fn from(e: &Vm) -> Self {
impl VmConfig {
    pub async fn from(e: &Vm) -> Result<Self, VirshleError> {
        // Todo(): make those values dynamic
        let kernel = "/run/cloud-hypervisor/hypervisor-fw";
        let mem_size = e.vram * u64::pow(1024, 3);
        let mut config = VmConfig {
            cpus: CpusConfig {
                boot_vcpus: e.vcpu,
                max_vcpus: e.vcpu * 2,
                ..Default::default()
            },
            memory: MemoryConfig {
                size: mem_size,
                shared: false,
                hugepages: true,
                // hugepage_size: Some(2048),
                // hotplug_size: Some(mem_size),
                ..Default::default()
            },
            disks: None,
            net: None,
            ..Default::default()
        };

        // Memory management
        let balloon = BalloonConfig {
            size: Some(e.vram / 2),
            deflate_on_oom: Some(true),
            free_page_reporting: Some(true),
        };
        config.balloon = Some(balloon);

        // Add bootloader
        let payload = PayloadConfig {
            kernel: Some("/run/cloud-hypervisor/hypervisor-fw".to_owned()),
            cmdline: Some(
                "earlyprintk=ttyS0 console=ttyS0 console=hvc0 root=/dev/vda1 rw".to_owned(),
            ),
        };
        config.payload = Some(payload);

        // Attach/Detach from standard output.
        // let console_socket = format!("{}/console.sock", e.get_dir()?);
        config.serial = Some(ConsoleConfig {
            // mode: ConsoleOutputMode::Socket,
            // socket: Some(console_socket),
            // mode: ConsoleOutputMode::Null,
            mode: ConsoleOutputMode::Tty,
            ..Default::default()
        });
        config.console = Some(ConsoleConfig {
            mode: ConsoleOutputMode::Null,
            // mode: ConsoleOutputMode::Socket,
            // mode: ConsoleOutputMode::Pty,
            ..Default::default()
        });

        let v_socket = format!("{}/ch.vsock", e.get_dir()?);
        if let Some(id) = e.id {
            let v_cid: u32 = format!("10{}", id).parse()?;
            config.vsock = Some(VsockConfig {
                cid: v_cid,
                socket: v_socket,
            });
        }

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
                let port_name = format!("vm-{}--{}", e.name, net.name);

                // Get fake_dhcp ip
                let mut ip: Option<IpAddr> = None;
                let mut mask: Option<IpAddr> = None;
                match VirshleConfig::get()?.dhcp {
                    Some(DhcpType::Fake(fake_dhcp)) => {
                        if let Some(pool) = fake_dhcp.pool.get(&net.name) {
                            ip = Some(pool.get_random_unleased_ip().await?);
                            mask = Some(pool.get_mask()?);
                        }
                    }
                    _ => {}
                }

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

                            // Fake dhcp
                            ip,
                            mask,

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
        // config
        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cloud_hypervisor::VmTemplate;
    use miette::{IntoDiagnostic, Result};
    use std::path::PathBuf;

    #[tokio::test]
    async fn make_vm_from_template() -> Result<()> {
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

        let vm = Vm::from(&vm_template)?;
        println!("{:#?}", vm);

        let vmm_config = VmConfig::from(&vm).await?;

        println!("{:#?}", vmm_config);
        Ok(())
    }

    #[tokio::test]
    async fn make_vm_from_definition_with_ids() -> Result<()> {
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
        let vmm_config = VmConfig::from(&vm).await?;

        // println!("{:#?}", vmm_config);
        Ok(())
    }
}
