use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, Value::Array};
use std::collections::HashMap;
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

use crate::network::InterfaceState;
mod json;

#[derive(Clone, Debug)]
pub struct Ovs;

// pub struct OvsResponse {
//     bridges: Vec<Bridge>,
// }

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct OvsInterface {
    #[serde(rename = "_uuid")]
    pub uuid: Uuid,
    #[serde(rename = "ifindex")]
    pub index: Option<u64>,
    pub name: String,
    #[serde(rename = "mac_in_use")]
    pub mac: String,
    #[serde(rename = "admin_state")]
    pub state: String,
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

pub enum OvsValue {
    String,
    Map,
    Set,
    Bool,
}

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
impl OvsPort {
    pub fn hydrate(&mut self) -> Result<(), VirshleError> {
        let uuid = self._interface_uuid;
        let interface = Ovs::get_interface_by_uuid(&uuid)?;
        self.interface = interface;
        Ok(())
    }
}

impl Ovs {
    /*
     * Split host main network interface to provide connectivity to vms.
     * see: ./README.md
     */
    pub fn make_host_switch() -> Result<(), VirshleError> {
        // Find main interface
        Ok(())
    }

    // Bridges
    pub fn get_bridges() -> Result<Vec<OvsBridge>, VirshleError> {
        let cmd = "sudo ovs-vsctl -f json list bridge".to_owned();

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

    pub fn get_ports() -> Result<Vec<OvsPort>, VirshleError> {
        let cmd = "sudo ovs-vsctl -f json list port".to_owned();

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
        let cmd = format!("sudo ovs-vsctl -f json list port {uuid}");

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
        let cmd = "sudo ovs-vsctl -f json list interface".to_owned();

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        let mut bridges = vec![];
        if let Some(stdout) = res.io.stdout {
            bridges = serde_json::from_value(Self::to_json(&stdout)?)?;
        }
        Ok(bridges)
    }
    pub fn get_interface_by_uuid(uuid: &Uuid) -> Result<OvsInterface, VirshleError> {
        let cmd = format!("sudo ovs-vsctl -f json list interface {uuid}");

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
        println!("{:#?}", res);
        Ok(())
    }
    // Ports
    #[test]
    fn test_ovs_get_ports() -> Result<()> {
        let res = Ovs::get_ports();
        println!("{:#?}", res);
        Ok(())
    }
    // Interfaces
    #[test]
    fn test_ovs_get_interfaces() -> Result<()> {
        let res = Ovs::get_interfaces();
        println!("{:#?}", res);
        Ok(())
    }

    // Create main switch.
    // #[test]
    fn test_ovs_make_patch() -> Result<()> {
        // ovs.make_host_switch()?;
        Ok(())
    }
    // #[test]
    // fn test_ip_get_main_switch() -> Result<()> {
    //     let res = Ovs::get_main_switch()?;
    //
    //     println!("{:#?}", res);
    //     Ok(())
    // }
}
