use serde::{Deserialize, Serialize};
use std::str::FromStr;
// Network primitives
use macaddr::MacAddr8;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
// Error handling
use log::info;
use miette::{IntoDiagnostic, Result};
use pipelight_exec::Process;
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Interface {
    #[serde(alias = "ifindex")]
    pub index: u64,
    #[serde(alias = "ifname")]
    pub name: String,
    #[serde(alias = "address")]
    pub mac: Option<String>,
    // #[serde(alias = "address")]
    // pub ips: Vec<IpAddr>,
}

#[derive(Default, Clone, Debug)]
pub struct Ip;

impl Ip {
    pub fn get_interfaces(&self) -> Result<Vec<Interface>, VirshleError> {
        let cmd = "ip -j a".to_owned();
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        let mut interfaces: Vec<Interface> = vec![];
        if let Some(stdout) = res.io.stdout {
            interfaces = serde_json::from_str(&stdout)?;
        }
        Ok(interfaces)
    }
}

#[derive(Default, Clone, Debug)]
pub struct Ovs;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Bridge {
    name: String,
    ports: Vec<String>,
}

impl Ovs {
    pub fn get_bridges(&self) -> Result<Vec<Bridge>, VirshleError> {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ip_get_interfaces() -> Result<()> {
        let ip = Ip::default();
        let res = ip.get_interfaces()?;

        println!("{:#?}", res);
        Ok(())
    }

    #[test]
    fn test_ovs_get_interface() -> Result<()> {
        let ovs = Ovs::default();
        let res = ovs.get_bridges();
        Ok(())
    }
}
