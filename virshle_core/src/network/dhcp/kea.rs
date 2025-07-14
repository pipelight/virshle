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
use ipnet::{
    IpAddrRange, IpNet, IpSub, IpSubnets, Ipv4AddrRange, Ipv6AddrRange, Ipv6Net, Ipv6Subnets,
};
use macaddr::{MacAddr, MacAddr6};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::IpPool;
use crate::connection::{Connection, ConnectionHandle, TcpConnection};
use crate::http_request::{Rest, RestClient};
use std::collections::HashMap;

// Error handling
use log::{error, trace};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeaDhcp {
    pub url: String,
    pub pool: Option<HashMap<String, IpPool>>,
}

/*
* Kea REST API types
*/
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct RestResponse {
    arguments: RestLeasesResponse,
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

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Lease {
    pub address: IpAddr,
    pub hostname: String,
    hwaddr: String,
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
            hwaddr: e.hwaddr.clone(), // hwaddr: MacAddr::V6(MacAddr6::from_str(&e.hwaddr)),
        }
    }
}
impl From<&Raw4Lease> for Lease {
    fn from(e: &Raw4Lease) -> Self {
        Lease {
            address: IpAddr::V4(e.address),
            hostname: e.hostname.clone(),
            hwaddr: e.hwaddr.clone(), // hwaddr: MacAddr::V6(MacAddr6::from_str(&e.hwaddr)),
        }
    }
}

pub const LEASES_DIR: &'static str = "/var/lib/kea";

#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct KeaCommand {
    command: String,
    service: Vec<String>,
    arguments: Option<HashMap<String, String>>,
}
#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct KeaBulkCommand {
    command: String,
    service: Vec<String>,
    arguments: Option<HashMap<String, HashMap<String, String>>>,
}

impl KeaDhcp {
    pub async fn get_leases_by_hostname(hostname: &str) -> Result<Vec<Lease>, VirshleError> {
        let mut leases: Vec<Lease> = Self::get_ipv6_leases_by_hostname(hostname).await?;
        leases.extend(Self::get_ipv4_leases_by_hostname(hostname).await?);
        Ok(leases)
    }
    pub async fn get_ipv6_leases_by_hostname(hostname: &str) -> Result<Vec<Lease>, VirshleError> {
        let mut conn = Connection::TcpConnection(TcpConnection::new("tcp://localhost:5547")?);
        let mut rest = RestClient::from(&mut conn);
        rest.open().await?;

        let cmd = KeaCommand {
            command: "lease6-get-by-hostname".to_owned(),
            service: vec!["dhcp6".to_owned()],
            arguments: Some(HashMap::from([(
                "hostname".to_owned(),
                format!("{}.", hostname),
            )])),
        };

        let mut leases: Vec<Lease> = vec![];
        let response: Vec<RestResponse> =
            rest.post("/", Some(cmd.clone())).await?.to_value().await?;
        if let Some(inside) = response.first() {
            leases = inside
                .arguments
                .leases
                .iter()
                .map(|e| Lease::from(e))
                .collect();
        }

        Ok(leases)
    }
    pub async fn get_ipv4_leases_by_hostname(hostname: &str) -> Result<Vec<Lease>, VirshleError> {
        let mut conn = Connection::TcpConnection(TcpConnection::new("tcp://localhost:5547")?);
        let mut rest = RestClient::from(&mut conn);
        rest.open().await?;

        let cmd = KeaCommand {
            command: "lease4-get-by-hostname".to_owned(),
            service: vec!["dhcp4".to_owned()],
            arguments: Some(HashMap::from([(
                "hostname".to_owned(),
                format!("{}", hostname),
            )])),
        };

        let mut leases: Vec<Lease> = vec![];
        let response: Vec<RestResponse> =
            rest.post("/", Some(cmd.clone())).await?.to_value().await?;
        if let Some(inside) = response.first() {
            leases = inside
                .arguments
                .leases
                .iter()
                .map(|e| Lease::from(e))
                .collect();
        }

        Ok(leases)
    }

    pub async fn get_leases() -> Result<Vec<Lease>, VirshleError> {
        let mut leases: Vec<Lease> = Self::get_ipv6_leases().await?;
        leases.extend(Self::get_ipv4_leases().await?);
        Ok(leases)
    }

    pub async fn get_ipv4_leases() -> Result<Vec<Lease>, VirshleError> {
        let mut conn = Connection::TcpConnection(TcpConnection::new("tcp://localhost:5547")?);
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
            leases = inside
                .arguments
                .leases
                .iter()
                .map(|e| Lease::from(e))
                .collect();
        }

        Ok(leases)
    }

    pub async fn get_ipv6_leases() -> Result<Vec<Lease>, VirshleError> {
        let mut conn = Connection::TcpConnection(TcpConnection::new("tcp://localhost:5547")?);
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
            leases = inside
                .arguments
                .leases
                .iter()
                .map(|e| Lease::from(e))
                .collect();
        }

        Ok(leases)
    }
    pub async fn delete_leases(vm_name: &str) -> Result<(), VirshleError> {
        Self::delete_ipv4_leases(vm_name).await?;
        Self::delete_ipv6_leases(vm_name).await?;
        Ok(())
    }
    pub async fn delete_ipv6_leases(vm_name: &str) -> Result<(), VirshleError> {
        let hostname = format!("vm-{}", vm_name);

        let mut conn = Connection::TcpConnection(TcpConnection::new("tcp://localhost:5547")?);
        let mut rest = RestClient::from(&mut conn);
        rest.open().await?;

        let mut ipv6_map: HashMap<String, String> = HashMap::new();
        let ipv6_leases = Self::get_ipv6_leases_by_hostname(&hostname).await?;
        for lease in ipv6_leases {
            ipv6_map.insert("ip-address".to_owned(), lease.address.to_string());
        }

        let cmd = KeaBulkCommand {
            command: "lease6-bulk-apply".to_owned(),
            service: vec!["dhcp6".to_owned()],
            arguments: Some(HashMap::from([("delete_leases".to_owned(), ipv6_map)])),
        };

        rest.post("/", Some(cmd.clone())).await?;
        Ok(())
    }
    pub async fn delete_ipv4_leases(vm_name: &str) -> Result<(), VirshleError> {
        let hostname = format!("vm-{}", vm_name);

        let mut conn = Connection::TcpConnection(TcpConnection::new("tcp://localhost:5547")?);
        let mut rest = RestClient::from(&mut conn);
        rest.open().await?;

        let mut ipv6_map: HashMap<String, String> = HashMap::new();
        let ipv6_leases = Self::get_ipv6_leases_by_hostname(&hostname).await?;
        for lease in ipv6_leases {
            ipv6_map.insert("ip-address".to_owned(), lease.address.to_string());
        }

        let cmd = KeaBulkCommand {
            command: "lease4-bulk-apply".to_owned(),
            service: vec!["dhcp4".to_owned()],
            arguments: Some(HashMap::from([("delete_leases".to_owned(), ipv6_map)])),
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
        let res = KeaDhcp::get_leases().await?;
        println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn read_leases4() -> Result<()> {
        let res = KeaDhcp::get_ipv4_leases().await?;
        println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn read_leases6() -> Result<()> {
        let res = KeaDhcp::get_ipv6_leases().await?;
        println!("{:#?}", res);
        Ok(())
    }
}
