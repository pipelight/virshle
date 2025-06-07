pub mod fd;
pub mod tap;

use crate::network::utils;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, Value::Array};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

// Network primitives
use macaddr::MacAddr8;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Error handling
use log::{error, info};
use miette::{IntoDiagnostic, Result};
use pipelight_exec::Process;
use virshle_error::{LibError, VirshleError, WrapError};

use super::InterfaceState;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct IpInterface {
    #[serde(rename = "ifindex")]
    pub index: Option<u64>,
    #[serde(rename = "ifname")]
    pub name: String,
    #[serde(rename = "address")]
    pub mac: Option<String>,
    #[serde(rename = "operstate")]
    pub state: String,
}

pub fn get_interfaces() -> Result<Vec<IpInterface>, VirshleError> {
    let cmd = "ip -j a".to_owned();
    let mut proc = Process::new();
    let res = proc.stdin(&cmd).run()?;

    let mut interfaces: Vec<IpInterface> = vec![];
    if let Some(stdout) = res.io.stdout {
        interfaces = serde_json::from_str(&stdout)?;
    }
    Ok(interfaces)
}

pub fn get_main_interface() -> Result<IpInterface, VirshleError> {
    let interfaces = get_interfaces()?;
    let main = interfaces.iter().find(|e| e.name.starts_with("en"));

    match main {
        None => {
            let message = "Couldn't find main ethernet interface.";
            let help = "Do you have eno1 or ens3..?";
            return Err(LibError::builder().msg(message).help(help).build().into());
        }
        Some(v) => return Ok(v.to_owned()),
    };
}

/*
* Bring interface up.
*/
pub fn up(name: &str) -> Result<(), VirshleError> {
    let name = utils::unix_name(name);

    #[cfg(debug_assertions)]
    let cmd = format!("sudo ip link set {} up", name);
    #[cfg(not(debug_assertions))]
    let cmd = format!("ip link set {} up", name);

    let mut proc = Process::new();
    let res = proc.stdin(&cmd).run()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    // Ip command
    #[test]
    fn test_ip_get_interfaces() -> Result<()> {
        let res = get_interfaces()?;

        println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn test_ip_get_main_interface() -> Result<()> {
        let res = get_main_interface()?;

        println!("{:#?}", res);
        Ok(())
    }
}
