use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, Value::Array};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

// Network primitives
use macaddr::MacAddr8;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Error handling
use log::info;
use miette::{IntoDiagnostic, Result};
use pipelight_exec::Process;
use virshle_error::{LibError, VirshleError, WrapError};

// Cloud-hypervisor
use crate::cloud_hypervisor::Vm;

//Fs
use std::fs;
use std::path::Path;

use crate::network::InterfaceState;
mod json;

#[derive(Clone, Debug)]
pub struct Ovs;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct OvsBridge {
    #[serde(rename = "_uuid")]
    pub uuid: Uuid,
    pub name: String,

    #[serde(rename = "ports")]
    _ports_uuid: Vec<Uuid>,
    #[serde(skip)]
    pub ports: Vec<OvsPort>,
}
impl OvsBridge {
    pub fn hydrate(&mut self) -> Result<(), VirshleError> {
        let mut ports: Vec<OvsPort> = vec![];
        for uuid in self._ports_uuid.clone() {
            let port = Ovs::get_port_by_uuid(&uuid)?;
            ports.push(port);
        }
        self.ports = ports;
        Ok(())
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct OvsPort {
    #[serde(rename = "_uuid")]
    pub uuid: Uuid,
    pub name: String,

    #[serde(rename = "interfaces")]
    _interface_uuid: Uuid,
    #[serde(skip)]
    pub interface: OvsInterface,
}
impl OvsPort {
    pub fn hydrate(&mut self) -> Result<(), VirshleError> {
        let uuid = self._interface_uuid;
        let interface = Ovs::get_interface_by_uuid(&uuid)?;
        self.interface = interface;
        Ok(())
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct OvsInterface {
    #[serde(rename = "_uuid")]
    pub uuid: Uuid,
    #[serde(default, rename = "type")]
    pub _type: Option<OvsInterfaceType>,
    #[serde(rename = "ifindex")]
    pub index: Option<u64>,
    pub name: String,
    #[serde(rename = "mac_in_use")]
    pub mac: String,
    #[serde(rename = "admin_state")]
    pub state: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OvsInterfaceType {
    #[default]
    System,
    Internal,
    Patch,
    DpdkVhostUserClient,
}
impl fmt::Display for OvsInterfaceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            OvsInterfaceType::Internal => "internal".to_owned(),
            OvsInterfaceType::Patch => "patch".to_owned(),
            OvsInterfaceType::DpdkVhostUserClient => "dpdkvhostuserclient".to_owned(),
            OvsInterfaceType::System => "system".to_owned(),
        };
        write!(f, "{}", string)
    }
}

impl Ovs {
    /*
     * Remove network port from the vm switch.
     */
    pub fn delete_vm_port(name: &str) -> Result<(), VirshleError> {
        let vm_bridge_name = Self::get_vm_bridge()?.name;

        let cmd = format!(
            "sudo ovs-vsctl \
            -- --if-exists del-port {vm_bridge_name} {name}"
        );
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        if let Some(stderr) = res.io.stderr {
            let message = "Ovs command failed.";
            let help = format!("{}\n{} ", stderr, &res.io.stdin.unwrap());
            return Err(LibError::new(message, &help).into());
        }
        Ok(())
    }
    /*
     * Add vm port into ovs config.
     */
    pub fn create_vm_port(name: &str, socket_path: &str) -> Result<(), VirshleError> {
        let vm_bridge_name = "br0";
        #[cfg(debug_assertions)]
        let cmd = format!(
            "sudo ovs-vsctl \
                -- --may-exist add-port {vm_bridge_name} {name} \
                -- set interface {name} type=dpdkvhostuserclient \
                -- set interface {name} options:vhost-server-path={socket_path} options:n_rxq=2"
        );
        #[cfg(not(debug_assertions))]
        let cmd = format!(
            "ovs-vsctl \
                -- --may-exist add-port {vm_bridge_name} {name} \
                -- set interface {name} type=dpdkvhostuserclient \
                -- set interface {name} options:vhost-server-path={socket_path} options:n_rxq=2"
        );
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        if let Some(stderr) = res.io.stderr {
            let message = "Ovs command failed";
            let help = format!("{}\n{} ", stderr, &res.io.stdin.unwrap());
            return Err(LibError::new(message, &help).into());
        }
        Ok(())
    }
    /*
     * Split host main network interface to provide connectivity to vms.
     * see: ./README.md
     */
    pub async fn ensure_switches() -> Result<(), VirshleError> {
        // Not fully implemented.
        // Consider there is already a main ovs switch on host
        // and link it to vm switch.

        // Will do the job for now.
        Self::_set_vm_bridge()?;
        Self::_clean_vm_bridge().await?;
        Ok(())
    }

    /*
     * Get ovs network switches/bridges.
     */
    pub fn get_bridges() -> Result<Vec<OvsBridge>, VirshleError> {
        #[cfg(debug_assertions)]
        let cmd = "sudo ovs-vsctl -f json list bridge".to_owned();
        #[cfg(not(debug_assertions))]
        let cmd = "ovs-vsctl -f json list bridge".to_owned();

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        match res.io.stdout {
            Some(v) => {
                let mut res: Vec<OvsBridge> = serde_json::from_value(Self::to_json(&v)?)?;
                // Hydration cascade
                res.iter_mut().for_each(|e| e.hydrate().unwrap());

                return Ok(res);
            }
            None => {
                let message = "Couldn't find any bridges";
                let help = "Do you have access right to ovs database?";
                return Err(LibError::new(message, help).into());
            }
        }
    }
    pub fn get_main_bridge() -> Result<OvsBridge, VirshleError> {
        let bridges = Self::get_bridges()?;
        let bridge = bridges.iter().find(|e| {
            e.ports
                .iter()
                .find(|e| e.interface.name.starts_with("en"))
                .is_some()
        });
        match bridge {
            Some(v) => Ok(v.to_owned()),
            None => {
                let message = "Couldn't identify the main bridge";
                let help = "Did you set up a main virtual switch correctly?";
                return Err(LibError::new(message, help).into());
            }
        }
    }
    pub fn get_vm_bridge() -> Result<OvsBridge, VirshleError> {
        let bridges = Self::get_bridges()?;
        let bridge = bridges.iter().find(|e| {
            e.ports
                .iter()
                .find(|e| {
                    e.interface.name.starts_with("patch")
                        && e.interface
                            .name
                            .ends_with(&Self::get_main_bridge().unwrap().name)
                })
                .is_some()
        });
        match bridge {
            Some(v) => Ok(v.to_owned()),
            None => {
                let message = "Couldn't identify the vm dedicated bridge";
                let help = "Did you set up a vm virtual switch correctly?";
                return Err(LibError::new(message, help).into());
            }
        }
    }
    /*
     * Create a virtual switch/bridge on host
     * to plug vms network main port in.
     */
    pub fn _set_vm_bridge() -> Result<(), VirshleError> {
        let vm_bridge_name = "br0";
        let main_bridge_name = Self::get_main_bridge()?.name;

        // - add patch cable to main switch (1/2)
        // - add vm switch
        // - add patch cable to vm switch (2/2)
        #[cfg(debug_assertions)]
        let cmd = format!(
            "sudo ovs-vsctl \
            -- --may-exist add-port {main_bridge_name} patch_{main_bridge_name}_{vm_bridge_name} \
            -- set interface patch_{main_bridge_name}_{vm_bridge_name} type=patch options:peer=patch_{vm_bridge_name}_{main_bridge_name} \
            -- --may-exist add-br {vm_bridge_name} \
            -- set bridge {vm_bridge_name} datapath_type=netdev \
            -- --may-exist add-port {vm_bridge_name} patch_{vm_bridge_name}_{main_bridge_name} \
            -- set interface patch_{vm_bridge_name}_{main_bridge_name} type=patch options:peer=patch_{main_bridge_name}_{vm_bridge_name}"
        );
        #[cfg(not(debug_assertions))]
        let cmd = format!(
            "ovs-vsctl \
            -- --may-exist add-port {main_bridge_name} patch_{main_bridge_name}_{vm_bridge_name} \
            -- set interface patch_{main_bridge_name}_{vm_bridge_name} type=patch options:peer=patch_{vm_bridge_name}_{main_bridge_name} \
            -- --may-exist add-br {vm_bridge_name} \
            -- set bridge {vm_bridge_name} datapath_type=netdev \
            -- --may-exist add-port {vm_bridge_name} patch_{vm_bridge_name}_{main_bridge_name} \
            -- set interface patch_{vm_bridge_name}_{main_bridge_name} type=patch options:peer=patch_{main_bridge_name}_{vm_bridge_name}"
        );

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        if let Some(stderr) = res.io.stderr {
            let message = "Ovs command failed";
            let help = format!("{}\n{} ", stderr, &res.io.stdin.unwrap());

            return Err(LibError::new(message, &help).into());
        }

        Ok(())
    }
    /*
     * Remove dpdkvhostuserclient ports from vm brigde
     * if not related to an existing vm in database.
     */
    pub async fn _clean_vm_bridge() -> Result<(), VirshleError> {
        let vms_name: Vec<String> = Vm::get_all()
            .await?
            .iter()
            .map(|e| e.name.to_owned())
            .collect();
        let vm_bridge = Ovs::get_vm_bridge()?;

        let mut cmd = format!("sudo ovs-vsctl");
        for port in vm_bridge.ports {
            if !vms_name.contains(&port.name) {
                if let Some(_type) = &port.interface._type {
                    match _type {
                        OvsInterfaceType::DpdkVhostUserClient => {
                            cmd += &format!(
                                " -- --if-exists del-port {} {}",
                                vm_bridge.name, port.name
                            );
                        }
                        _ => {}
                    };
                }
            }
        }
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;
        Ok(())
    }

    pub fn get_ports() -> Result<Vec<OvsPort>, VirshleError> {
        #[cfg(debug_assertions)]
        let cmd = "sudo ovs-vsctl -f json list port".to_owned();
        #[cfg(not(debug_assertions))]
        let cmd = "ovs-vsctl -f json list port".to_owned();

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        match res.io.stdout {
            Some(v) => {
                let mut res: Vec<OvsPort> = serde_json::from_value(Self::to_json(&v)?)?;
                // Hydration cascade
                res.iter_mut().for_each(|e| e.hydrate().unwrap());
                return Ok(res);
            }
            None => {
                let message = "Couldn't find any ports";
                let help = "Do you have access right to ovs database?";
                return Err(LibError::new(message, help).into());
            }
        }
    }

    pub fn get_port_by_uuid(uuid: &Uuid) -> Result<OvsPort, VirshleError> {
        #[cfg(debug_assertions)]
        let cmd = format!("sudo ovs-vsctl -f json list port {uuid}");
        #[cfg(not(debug_assertions))]
        let cmd = format!("ovs-vsctl -f json list port {uuid}");

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        match res.io.stdout {
            Some(v) => match Self::to_json(&v)?.as_array().unwrap().first() {
                Some(v) => {
                    let mut res: OvsPort = serde_json::from_value(v.to_owned())?;
                    res.hydrate()?;
                    return Ok(res);
                }
                None => {
                    let message = format!("Couldn't find a port with uuid: {uuid}");
                    let help = "Are you sure this port exists?";
                    return Err(LibError::new(&message, help).into());
                }
            },
            None => {
                let message = format!("Couldn't find a port with uuid: {uuid}");
                let help = "Do you have access right to ovs database?";
                return Err(LibError::new(&message, help).into());
            }
        }
    }
    /*
     * Return the bridge that is linked to
     * the main network interface/ ethernet port (eno1)
     */
    // pub fn get_main_switch(&self) -> Result<Bridge, VirshleError> {
    //     let bridges = self.get_bridges()?;
    //     Ok(bridge)
    // }
    pub fn get_interfaces() -> Result<Vec<OvsInterface>, VirshleError> {
        #[cfg(debug_assertions)]
        let cmd = "sudo ovs-vsctl -f json list interface".to_owned();
        #[cfg(not(debug_assertions))]
        let cmd = "ovs-vsctl -f json list interface".to_owned();

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        let mut bridges = vec![];
        if let Some(stdout) = res.io.stdout {
            bridges = serde_json::from_value(Self::to_json(&stdout)?)?;
        }
        Ok(bridges)
    }
    pub fn get_interface_by_uuid(uuid: &Uuid) -> Result<OvsInterface, VirshleError> {
        #[cfg(debug_assertions)]
        let cmd = format!("sudo ovs-vsctl -f json list interface {uuid}");
        #[cfg(not(debug_assertions))]
        let cmd = format!("ovs-vsctl -f json list interface {uuid}");

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        match res.io.stdout {
            Some(v) => match Self::to_json(&v)?.as_array().unwrap().first() {
                Some(v) => {
                    let res: OvsInterface = serde_json::from_value(v.to_owned())?;
                    return Ok(res);
                }
                None => {
                    let message = format!("Couldn't find an interface with uuid: {uuid}");
                    let help = "Are you sure this interface exists?";
                    return Err(LibError::new(&message, help).into());
                }
            },
            None => {
                let message = format!("Couldn't find an interface with uuid: {uuid}");
                let help = "Do you have access right to ovs database?";
                return Err(LibError::new(&message, help).into());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Brigdges
    #[test]
    fn test_ovs_get_bridges() -> Result<()> {
        let res = Ovs::get_bridges()?;
        // println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn test_ovs_get_main_bridge() -> Result<()> {
        let res = Ovs::get_main_bridge()?;
        // println!("{:#?}", res);
        Ok(())
    }
    // Ports
    #[test]
    fn test_ovs_get_ports() -> Result<()> {
        let res = Ovs::get_ports();
        // println!("{:#?}", res);
        Ok(())
    }
    // Interfaces
    #[test]
    fn test_ovs_get_interfaces() -> Result<()> {
        let res = Ovs::get_interfaces();
        // println!("{:#?}", res);
        Ok(())
    }

    // Create main switch.
    #[tokio::test]
    async fn test_ovs_config_host() -> Result<()> {
        Ovs::ensure_switches().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_clean_vm_bridge() -> Result<()> {
        Ovs::_clean_vm_bridge().await?;
        Ok(())
    }
}
