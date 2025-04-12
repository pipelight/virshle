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

#[derive(Default, Clone, Debug)]
pub struct Ip;

impl Ip {
    pub fn get_interfaces(&self) -> Result<Vec<IpInterface>, VirshleError> {
        let cmd = "ip -j a".to_owned();
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        let mut interfaces: Vec<IpInterface> = vec![];
        if let Some(stdout) = res.io.stdout {
            interfaces = serde_json::from_str(&stdout)?;
        }
        Ok(interfaces)
    }
    pub fn get_main_interface(&self) -> Result<IpInterface, VirshleError> {
        let interfaces = self.get_interfaces()?;
        let main = interfaces.iter().find(|e| e.name.starts_with("en"));

        match main {
            None => {
                let message = "Couldn't find main ethernet interface.";
                let help = "Do you have eno1 or ens3..?";
                return Err(LibError::new(message, help).into());
            }
            Some(v) => return Ok(v.to_owned()),
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Ip command
    #[test]
    fn test_ip_get_interfaces() -> Result<()> {
        let ip = Ip::default();
        let res = ip.get_interfaces()?;

        println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn test_ip_get_main_interface() -> Result<()> {
        let ip = Ip::default();
        let res = ip.get_main_interface()?;

        println!("{:#?}", res);
        Ok(())
    }
}
