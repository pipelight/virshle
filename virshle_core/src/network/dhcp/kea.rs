use axum::response;
// Files

use csv;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

// Time
use jiff::Timestamp;

// IP
use super::Lease;
use ipnet::{
    IpAddrRange, IpNet, IpSub, IpSubnets, Ipv4AddrRange, Ipv6AddrRange, Ipv6Net, Ipv6Subnets,
};
use macaddr::{MacAddr, MacAddr6};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::IpPool;
use crate::connection::{Connection, ConnectionHandle, TcpConnection};
use crate::http_request::{Rest, RestClient};
use crate::Vm;
use std::collections::HashMap;

// Error handling
use log::{error, trace};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeaDhcp {
    pub url: Option<String>,
    pub suffix: Option<String>,
}
impl Default for KeaDhcp {
    fn default() -> Self {
        Self {
            url: Some("tcp://localhost:5547".to_owned()),
            suffix: Some("vm".to_owned()),
        }
    }
}

/*
* Kea REST API types
*/
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct RestResponse {
    arguments: Option<RestLeasesResponse>,
    result: u64,
}
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct RestLeasesResponse {
    leases: Vec<RawLease>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RawLease {
    V4(Raw4Lease),
    V6(Raw6Lease),
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Raw4Lease {
    #[serde(rename = "ip-address")]
    address: Ipv4Addr,
    #[serde(rename = "hw-address")]
    hwaddr: String, // MacAddr
    #[serde(skip)]
    client_id: String,
    #[serde(rename = "valid-lft")]
    valid_lifetime: u64,
    hostname: String,
    state: u64,
}
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Raw6Lease {
    #[serde(rename = "ip-address")]
    address: Ipv6Addr,
    #[serde(rename = "hw-address")]
    hwaddr: String, // MacAddr
    #[serde(skip)]
    client_id: String,
    #[serde(rename = "valid-lft")]
    valid_lifetime: u64,
    #[serde(rename = "type")]
    _type: String,
    hostname: String,
    state: u64,
}

impl From<&RawLease> for Lease {
    fn from(e: &RawLease) -> Self {
        match e {
            RawLease::V6(v) => v.into(),
            RawLease::V4(v) => v.into(),
        }
    }
}
impl From<&Raw6Lease> for Lease {
    fn from(e: &Raw6Lease) -> Self {
        let hostname = e.hostname.strip_suffix(".").unwrap().to_owned();
        Lease {
            address: IpAddr::V6(e.address),
            hostname,
            mac: MacAddr6::from_str(&e.hwaddr).unwrap(),
        }
    }
}
impl From<&Raw4Lease> for Lease {
    fn from(e: &Raw4Lease) -> Self {
        Lease {
            address: IpAddr::V4(e.address),
            hostname: e.hostname.clone(),
            mac: MacAddr6::from_str(&e.hwaddr).unwrap(),
        }
    }
}

pub const LEASES_DIR: &'static str = "/var/lib/kea";

#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct KeaCommand {
    command: String,
    service: Vec<String>,
    #[serde(skip)]
    arguments: Option<HashMap<String, String>>,
}
#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct KeaBulkCommand {
    command: String,
    service: Vec<String>,
    #[serde(skip)]
    arguments: Option<HashMap<String, HashMap<String, String>>>,
}

impl KeaDhcp {
    /// Get domain name from ip address type and vm_name.
    pub fn to_domain_name(&self, ip_type: IpAddr, vm_name: &str) -> Result<String, VirshleError> {
        let mut domain = vm_name.to_owned();
        if let Some(suffix) = &self.suffix {
            domain += ".";
            domain += suffix;
        }
        match ip_type {
            IpAddr::V4(_) => {}
            IpAddr::V6(_) => {
                domain += ".";
            }
        };
        Ok(domain)
    }
    /// Get original vm name from dhcp record.
    pub fn to_vm_name(&self, lease: &Lease) -> Result<String, VirshleError> {
        let vm_name = match lease.address {
            IpAddr::V4(_) => {
                let mut name = lease.hostname.clone();
                if let Some(suffix) = &self.suffix {
                    name = name.strip_suffix(suffix).unwrap().to_owned();
                }
                name.trim_end_matches('.');
                name
            }
            IpAddr::V6(_) => {
                let mut name = lease.hostname.clone();
                if let Some(suffix) = &self.suffix {
                    name = name.strip_suffix(suffix).unwrap().to_owned();
                }
                name.trim_end_matches('.');
                name
            }
        };
        Ok(vm_name)
    }
}

impl KeaDhcp {
    pub async fn get_leases_by_hostname(&self, hostname: &str) -> Result<Vec<Lease>, VirshleError> {
        let mut leases: Vec<Lease> = self.get_ipv6_leases_by_hostname(hostname).await?;
        leases.extend(self.get_ipv4_leases_by_hostname(hostname).await?);
        Ok(leases)
    }
    pub async fn get_ipv6_leases_by_hostname(
        &self,
        vm_name: &str,
    ) -> Result<Vec<Lease>, VirshleError> {
        let mut conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest = RestClient::from(&mut conn);
        rest.open().await?;

        let hostname = self.to_domain_name(IpAddr::V6(Ipv6Addr::UNSPECIFIED), vm_name)?;
        let cmd = KeaCommand {
            command: "lease6-get-by-hostname".to_owned(),
            service: vec!["dhcp6".to_owned()],
            arguments: Some(HashMap::from([("hostname".to_owned(), hostname)])),
        };

        let mut leases: Vec<Lease> = vec![];
        let response: Vec<RestResponse> =
            rest.post("/", Some(cmd.clone())).await?.to_value().await?;
        if let Some(inside) = response.first() {
            if let Some(arguments) = &inside.arguments {
                leases = arguments.leases.iter().map(|e| Lease::from(e)).collect();
            }
        }

        Ok(leases)
    }
    pub async fn get_ipv4_leases_by_hostname(
        &self,
        vm_name: &str,
    ) -> Result<Vec<Lease>, VirshleError> {
        let mut conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest = RestClient::from(&mut conn);
        rest.open().await?;

        let hostname = self.to_domain_name(IpAddr::V4(Ipv4Addr::UNSPECIFIED), vm_name)?;
        let cmd = KeaCommand {
            command: "lease4-get-by-hostname".to_owned(),
            service: vec!["dhcp4".to_owned()],
            arguments: Some(HashMap::from([("hostname".to_owned(), hostname)])),
        };

        let mut leases: Vec<Lease> = vec![];
        let response: Vec<RestResponse> =
            rest.post("/", Some(cmd.clone())).await?.to_value().await?;
        if let Some(inside) = response.first() {
            if let Some(arguments) = &inside.arguments {
                leases = arguments.leases.iter().map(|e| Lease::from(e)).collect();
            }
        }

        Ok(leases)
    }

    pub async fn get_leases(&self) -> Result<Vec<Lease>, VirshleError> {
        let mut leases: Vec<Lease> = self.get_ipv6_leases().await?;
        leases.extend(self.get_ipv4_leases().await?);
        Ok(leases)
    }

    pub async fn get_ipv4_leases(&self) -> Result<Vec<Lease>, VirshleError> {
        let mut conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest = RestClient::from(&mut conn);
        rest.open().await?;

        let cmd = KeaCommand {
            command: "lease4-get-all".to_owned(),
            service: vec!["dhcp4".to_owned()],
            ..Default::default()
        };

        let mut leases: Vec<Lease> = vec![];
        let response: Vec<RestResponse> =
            rest.post("/", Some(cmd.clone())).await?.to_value().await?;
        if let Some(inside) = response.first() {
            if let Some(arguments) = &inside.arguments {
                leases = arguments.leases.iter().map(|e| Lease::from(e)).collect();
            }
        }

        Ok(leases)
    }

    pub async fn get_ipv6_leases(&self) -> Result<Vec<Lease>, VirshleError> {
        let mut conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest = RestClient::from(&mut conn);
        rest.open().await?;

        let cmd = KeaCommand {
            command: "lease6-get-all".to_owned(),
            service: vec!["dhcp6".to_owned()],
            ..Default::default()
        };

        let mut leases: Vec<Lease> = vec![];
        let response: Vec<RestResponse> =
            rest.post("/", Some(cmd.clone())).await?.to_value().await?;
        if let Some(inside) = response.first() {
            if let Some(arguments) = &inside.arguments {
                leases = arguments.leases.iter().map(|e| Lease::from(e)).collect();
            }
        }

        Ok(leases)
    }

    pub async fn clean_leases(&self) -> Result<(), VirshleError> {
        self.clean_ipv6_leases().await?;
        self.clean_ipv4_leases().await?;
        Ok(())
    }
    /// Remove leases if associated vm doesn't exist.
    pub async fn clean_ipv4_leases(&self) -> Result<(), VirshleError> {
        // Get vms
        let vms: Vec<String> = Vm::get_all()
            .await?
            .iter()
            .map(|e| e.name.clone())
            .collect();

        // Get leases
        let mut leases = self.get_ipv4_leases().await?;

        // Remove leases if no corresponding vm name
        leases = leases
            .iter()
            .filter(|e| vms.contains(&self.to_vm_name(e).unwrap()))
            .map(|e| e.to_owned())
            .collect();

        self.delete_ipv6_leases(leases).await?;
        Ok(())
    }
    /// Remove leases if associated vm doesn't exist.
    pub async fn clean_ipv6_leases(&self) -> Result<(), VirshleError> {
        // Get vms
        let vms: Vec<String> = Vm::get_all()
            .await?
            .iter()
            .map(|e| e.name.clone())
            .collect();

        // Get leases
        let mut leases = self.get_ipv6_leases().await?;

        // Remove leases if no corresponding vm name
        leases = leases
            .iter()
            .filter(|e| vms.contains(&self.to_vm_name(e).unwrap()))
            .map(|e| e.to_owned())
            .collect();

        self.delete_ipv6_leases(leases).await?;
        Ok(())
    }

    pub async fn delete_leases(&self, vm_name: &str) -> Result<(), VirshleError> {
        self.delete_ipv4_leases_by_name(vm_name).await?;
        self.delete_ipv6_leases_by_name(vm_name).await?;
        Ok(())
    }

    // Delete a list of leases.
    pub async fn delete_ipv6_leases(&self, leases: Vec<Lease>) -> Result<(), VirshleError> {
        let mut conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest = RestClient::from(&mut conn);
        rest.open().await?;

        let mut req_map: HashMap<String, String> = HashMap::new();
        for lease in leases {
            req_map.insert("ip-address".to_owned(), lease.address.to_string());
        }
        let cmd = KeaBulkCommand {
            command: "lease6-bulk-apply".to_owned(),
            service: vec!["dhcp6".to_owned()],
            arguments: Some(HashMap::from([("delete_leases".to_owned(), req_map)])),
        };

        rest.post("/", Some(cmd.clone())).await?;
        Ok(())
    }

    // Delete a vm leases.
    pub async fn delete_ipv6_leases_by_name(&self, vm_name: &str) -> Result<(), VirshleError> {
        let mut conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest = RestClient::from(&mut conn);
        rest.open().await?;

        let hostname = self.to_domain_name(IpAddr::V6(Ipv6Addr::UNSPECIFIED), vm_name)?;

        let mut req_map: HashMap<String, String> = HashMap::new();
        let leases = self.get_ipv6_leases_by_hostname(&hostname).await?;
        for lease in leases {
            req_map.insert("ip-address".to_owned(), lease.address.to_string());
        }

        let cmd = KeaBulkCommand {
            command: "lease6-bulk-apply".to_owned(),
            service: vec!["dhcp6".to_owned()],
            arguments: Some(HashMap::from([("delete_leases".to_owned(), req_map)])),
        };

        rest.post("/", Some(cmd.clone())).await?;
        Ok(())
    }

    // Delete a vm leases.
    pub async fn delete_ipv4_leases_by_name(&self, vm_name: &str) -> Result<(), VirshleError> {
        let mut conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest = RestClient::from(&mut conn);
        rest.open().await?;

        let hostname = self.to_domain_name(IpAddr::V4(Ipv4Addr::UNSPECIFIED), vm_name)?;

        let mut req_map: HashMap<String, String> = HashMap::new();
        let leases = self.get_ipv4_leases_by_hostname(&hostname).await?;
        for lease in leases {
            req_map.insert("ip-address".to_owned(), lease.address.to_string());
        }

        let cmd = KeaBulkCommand {
            command: "lease4-bulk-apply".to_owned(),
            service: vec!["dhcp4".to_owned()],
            arguments: Some(HashMap::from([("delete_leases".to_owned(), req_map)])),
        };

        rest.post("/", Some(cmd.clone())).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn read_leases() -> Result<()> {
        let res = KeaDhcp::default().get_leases().await?;
        println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn read_leases4() -> Result<()> {
        let res = KeaDhcp::default().get_ipv4_leases().await?;
        println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn read_leases6() -> Result<()> {
        let res = KeaDhcp::default().get_ipv6_leases().await?;
        println!("{:#?}", res);
        Ok(())
    }
}
