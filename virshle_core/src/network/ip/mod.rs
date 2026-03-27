pub mod fd;
pub mod macvtap;
pub mod tap;

use super::utils;

use bon::builder;
use libc::mntent;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, Value::Array};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

// Network primitives
use macaddr::MacAddr8;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Error handling
use miette::{IntoDiagnostic, Result};
use pipelight_exec::Process;
use tracing::{error, info};
use virshle_error::{LibError, VirshleError, WrapError};

use super::InterfaceState;

/// Output from "ip link" and "ip address"
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct IpInterface {
    #[serde(rename = "ifindex")]
    pub index: u64,
    #[serde(rename = "ifname")]
    pub name: String,
    #[serde(rename = "address")]
    pub mac: Option<String>,
    #[serde(rename = "operstate")]
    pub state: String,
    #[serde(rename = "addr_info")]
    pub ips: Option<Vec<AddrInfo>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AddrInfo {
    #[serde(rename = "local")]
    pub address: String,
    pub dynamic: Option<bool>,
    pub scope: Scope,
    // Ipv6 feat: Set to true if the address is managed by the kernel.
    pub mngtmpaddr: Option<bool>,
}
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    #[default]
    Host,
    Global,
    Local,
    Link,
}

pub fn get_interfaces() -> Result<Vec<IpInterface>, VirshleError> {
    let cmd = "ip --detail --json link".to_owned();
    let mut proc = Process::new();
    let res = proc.stdin(&cmd).run()?;

    let mut interfaces: Vec<IpInterface> = vec![];
    if let Some(stdout) = res.io.stdout {
        interfaces = serde_json::from_str(&stdout)?;
    }
    Ok(interfaces)
}

#[builder(
    finish_fn = exec, 
    on(_, into)
)]
pub fn get_ips(inet6: Option<bool>, inet4: Option<bool>, scope: Option<Scope>, prefix: Option<String>, mngtmpaddr: Option<bool>) -> Result<Vec<IpAddr>, VirshleError> {
    let cmd = "ip --detail --json address".to_owned();
    let mut proc = Process::new();
    let res = proc.stdin(&cmd).run()?;

    let mut interfaces: Vec<IpInterface> = vec![];
    if let Some(stdout) = res.io.stdout {
        interfaces = serde_json::from_str(&stdout)?;
    }
    // println!("{:#?}", interfaces);
    let mut addr: Vec<IpAddr> = vec![];
    for interface in interfaces {
        if let Some(ips) = interface.ips {
            for info in ips {
                if let Some(scope) = &scope {
                    if scope != &info.scope {
                        continue;
                    }
                }
                if let Some(mngtmpaddr) = mngtmpaddr {
                    if mngtmpaddr != info.mngtmpaddr.unwrap_or(false) {
                        continue;
                    }
                }
                if let Some(prefix) = &prefix {
                    if !info.address.starts_with(prefix) {
                        continue;
                    }
                }
                let e = IpAddr::from_str(&info.address).unwrap();
                if inet4 == Some(true) 
                || inet6 == Some(false) {
                    if e.is_ipv4() {
                        addr.push(e);
                    }
                }
                if inet6 == Some(true) 
                || inet4 == Some(false) {
                    if e.is_ipv6() {
                        addr.push(e);
                    }
                }
            }
        }
    }
    Ok(addr)
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
    fn _get_interfaces() -> Result<()> {
        let res = get_interfaces()?;
        println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn _get_main_interface() -> Result<()> {
        let res = get_main_interface()?;
        println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn _get_ip_addresses() -> Result<()> {
        // Get all ipv6.
        let res = get_ips().inet6(true).exec()?;
        println!("{:#?}", res);
        // Get all global ipv6.
        let res = get_ips().inet6(true).scope(Scope::Global).exec()?;
        println!("{:#?}", res);
        // Get the global incoming ipv6.
        let res = get_ips().inet6(true).scope(Scope::Global).mngtmpaddr(false).prefix("2").exec()?;
        println!("{:#?}", res);
        // Get all ipv4
        let res = get_ips().inet4(true).exec()?;
        println!("{:#?}", res);
        Ok(())
    }
}
