use super::convert;

use std::rc::Rc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, Value::Array};
use std::fmt;
use uuid::Uuid;

// Error handling
use log::{error, info};
use miette::{IntoDiagnostic, Result};
use pipelight_exec::Process;
use virshle_error::{LibError, VirshleError, WrapError};

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
        let bridge = Rc::new(self.to_owned());
        let ports_uuid = self._ports_uuid.clone();
        for uuid in ports_uuid {
            let mut port = OvsPort::get_by_uuid(uuid)?;
            port.bridge = Rc::clone(&bridge);
            self.ports.push(port);
        }
        Ok(())
    }

    pub fn get(name: &str) -> Result<OvsBridge, VirshleError> {
        #[cfg(debug_assertions)]
        let cmd = "sudo ovs-vsctl -f json list bridge".to_owned();
        #[cfg(not(debug_assertions))]
        let cmd = "ovs-vsctl -f json list bridge".to_owned();

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        if let Some(stdout) = res.io.stdout {
            let mut bridges: Vec<OvsBridge> = serde_json::from_value(convert::to_json(&stdout)?)?;
            bridges = bridges
                .iter()
                .filter(|e| e.name == name)
                .map(|e| e.to_owned())
                .collect();

            if let Some(bridge) = bridges.first_mut() {
                bridge.hydrate().unwrap();
                return Ok(bridge.to_owned());
            }
        }
        // Error
        let message = format!("Couldn't a bridge with name: {}", name);
        let help = "Do you have access right to the ovs database?";
        return Err(LibError::builder().msg(&message).help(help).build().into());
    }
    /*
     * Get ovs network switches/bridges.
     */
    pub fn get_all() -> Result<Vec<OvsBridge>, VirshleError> {
        #[cfg(debug_assertions)]
        let cmd = "sudo ovs-vsctl -f json list bridge".to_owned();
        #[cfg(not(debug_assertions))]
        let cmd = "ovs-vsctl -f json list bridge".to_owned();

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        match res.io.stdout {
            Some(v) => {
                let mut bridges: Vec<OvsBridge> = serde_json::from_value(convert::to_json(&v)?)?;
                // Hydration cascade
                bridges.iter_mut().for_each(|e| e.hydrate().unwrap());

                return Ok(bridges);
            }
            None => {
                let message = "Couldn't find any bridges";
                let help = "Do you have access right to ovs database?";
                return Err(LibError::builder().msg(message).help(help).build().into());
            }
        }
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

    #[serde(skip)]
    pub bridge: Rc<OvsBridge>,
}

impl OvsPort {
    pub fn hydrate(&mut self) -> Result<(), VirshleError> {
        let uuid = self._interface_uuid;
        let interface = OvsInterface::get_by_uuid(&uuid)?;
        self.interface = interface;
        Ok(())
    }
    pub fn get_all() -> Result<Vec<OvsPort>, VirshleError> {
        #[cfg(debug_assertions)]
        let cmd = "sudo ovs-vsctl -f json list port".to_owned();
        #[cfg(not(debug_assertions))]
        let cmd = "ovs-vsctl -f json list port".to_owned();

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        match res.io.stdout {
            Some(v) => {
                let mut res: Vec<OvsPort> = serde_json::from_value(convert::to_json(&v)?)?;
                // Hydration cascade
                res.iter_mut().for_each(|e| e.hydrate().unwrap());
                return Ok(res);
            }
            None => {
                let message = "Couldn't find any ports";
                let help = "Do you have access right to ovs database?";
                return Err(LibError::builder().msg(message).help(help).build().into());
            }
        }
    }
    pub fn get_by_uuid(uuid: Uuid) -> Result<OvsPort, VirshleError> {
        #[cfg(debug_assertions)]
        let cmd = format!("sudo ovs-vsctl -f json list port {uuid}");
        #[cfg(not(debug_assertions))]
        let cmd = format!("ovs-vsctl -f json list port {uuid}");

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        match res.io.stdout {
            Some(v) => match convert::to_json(&v)?.as_array().unwrap().first() {
                Some(v) => {
                    let mut res: OvsPort = serde_json::from_value(v.to_owned())?;
                    res.hydrate()?;
                    return Ok(res);
                }
                None => {
                    let message = format!("Couldn't find a port with uuid: {uuid}");
                    let help = "Are you sure this port exists?";
                    return Err(LibError::builder().msg(&message).help(help).build().into());
                }
            },
            None => {
                let message = format!("Couldn't find a port with uuid: {uuid}");
                let help = "Do you have access right to ovs database?";
                return Err(LibError::builder().msg(&message).help(help).build().into());
            }
        }
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

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OvsInterfaceType {
    #[default]
    System,
    Internal,
    Patch,
    DpdkVhostUserClient,
    Tap,
}
impl fmt::Display for OvsInterfaceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            OvsInterfaceType::System => "system".to_owned(),
            OvsInterfaceType::Internal => "internal".to_owned(),
            OvsInterfaceType::Patch => "patch".to_owned(),
            OvsInterfaceType::DpdkVhostUserClient => "dpdkvhostuserclient".to_owned(),
            OvsInterfaceType::Tap => "tap".to_owned(),
        };
        write!(f, "{}", string)
    }
}
impl OvsInterface {
    pub fn get_all() -> Result<Vec<OvsInterface>, VirshleError> {
        #[cfg(debug_assertions)]
        let cmd = "sudo ovs-vsctl -f json list interface".to_owned();
        #[cfg(not(debug_assertions))]
        let cmd = "ovs-vsctl -f json list interface".to_owned();

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        let mut bridges = vec![];
        if let Some(stdout) = res.io.stdout {
            bridges = serde_json::from_value(convert::to_json(&stdout)?)?;
        }
        Ok(bridges)
    }
    pub fn get_by_uuid(uuid: &Uuid) -> Result<OvsInterface, VirshleError> {
        #[cfg(debug_assertions)]
        let cmd = format!("sudo ovs-vsctl -f json list interface {uuid}");
        #[cfg(not(debug_assertions))]
        let cmd = format!("ovs-vsctl -f json list interface {uuid}");

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        match res.io.stdout {
            Some(v) => match convert::to_json(&v)?.as_array().unwrap().first() {
                Some(v) => {
                    let res: OvsInterface = serde_json::from_value(v.to_owned())?;
                    return Ok(res);
                }
                None => {
                    let message = format!("Couldn't find an interface with uuid: {uuid}");
                    let help = "Are you sure this interface exists?";
                    return Err(LibError::builder().msg(&message).help(help).build().into());
                }
            },
            None => {
                let message = format!("Couldn't find an interface with uuid: {uuid}");
                let help = "Do you have access right to ovs database?";
                return Err(LibError::builder().msg(&message).help(help).build().into());
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
        let res = OvsBridge::get_all()?;
        println!("{:#?}", res);
        Ok(())
    }
    // Ports
    #[test]
    fn test_ovs_get_ports() -> Result<()> {
        let res = OvsPort::get_all()?;
        println!("{:#?}", res);
        Ok(())
    }
    // Interfaces
    #[test]
    fn test_ovs_get_interfaces() -> Result<()> {
        let res = OvsInterface::get_all()?;
        println!("{:#?}", res);
        Ok(())
    }
}
