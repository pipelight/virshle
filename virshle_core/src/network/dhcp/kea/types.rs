// TODO: Refactor this fucking mess:
// factorize functions!!

use bon::{bon, builder};

use serde::{Deserialize, Serialize};
use std::str::FromStr;

// IP
use macaddr::MacAddr6;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::config::{Config, DhcpType, KeaDhcpConfig};
use crate::network::dhcp::Lease;

use virshle_network::{
    connection::{Connection, TcpConnection},
    http::{Rest, RestClient},
};

// Error handling
use miette::Result;
use virshle_error::VirshleError;

pub const LEASES_DIR: &'static str = "/var/lib/kea";

#[derive(Debug)]
pub struct KeaDhcp {
    pub url: Option<String>,
    pub suffix: Option<String>,
    pub rest: RestClient,
}
#[bon]
impl KeaDhcp {
    #[builder(
        start_fn = new,
        finish_fn = build
    )]
    // Create new struct and open a connection to kea-ctrl-agent http rest api.
    pub async fn _new(config: &Config) -> Result<KeaDhcp, VirshleError> {
        // Default config.
        let default_conf = KeaDhcpConfig {
            url: Some("tcp://localhost:5547".to_owned()),
            suffix: Some("vm".to_owned()),
        };
        let mut res = KeaDhcp::builder().config(&default_conf).build().await?;

        // Config from file.
        if let Some(config) = &config.dhcp {
            match config {
                DhcpType::Fake(_) => {}
                DhcpType::Kea(e) => {
                    res = KeaDhcp::builder().config(&e).build().await?;
                }
            }
        }
        Ok(res)
    }
    #[builder(
        finish_fn = build
    )]
    pub async fn builder(config: &KeaDhcpConfig) -> Result<KeaDhcp, VirshleError> {
        let conn = Connection::TcpConnection(TcpConnection::new(&config.url.clone().unwrap())?);
        let mut rest: RestClient = conn.into();
        rest.open().await?;
        let res = KeaDhcp {
            url: config.url.clone(),
            suffix: config.suffix.clone(),
            rest: rest,
        };
        Ok(res)
    }
}

// Kea REST API types
//
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct RestResponse {
    arguments: Option<RestLeasesResponse>,
    result: u64,
}
impl RestResponse {
    pub fn to_leases(response: Vec<RestResponse>) -> Result<Vec<Lease>, VirshleError> {
        let mut leases: Vec<Lease> = vec![];
        if let Some(inside) = response.first() {
            if let Some(arguments) = &inside.arguments {
                leases = arguments
                    .leases
                    .clone()
                    .into_iter()
                    .map(|e| e.into())
                    .collect();
            }
        }
        Ok(leases)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct RestLeasesResponse {
    leases: Vec<RawLease>,
}
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RawLease {
    V6(Raw6Lease),
    V4(Raw4Lease),
}
impl Into<Lease> for RawLease {
    fn into(self) -> Lease {
        (&self).into()
    }
}
impl Into<Lease> for &RawLease {
    fn into(self) -> Lease {
        match self {
            RawLease::V6(v) => v.into(),
            RawLease::V4(v) => v.into(),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Raw4Lease {
    #[serde(rename = "ip-address")]
    address: Ipv4Addr,
    #[serde(default, rename = "hw-address")]
    hwaddr: Option<String>, // MacAddr
    #[serde(rename = "valid-lft")]
    valid_lifetime: u64,
    #[serde(default)]
    hostname: Option<String>,
    state: u64,
    #[serde(rename = "subnet-id")]
    subnet_id: u64,
    #[serde(flatten)]
    other: serde_json::Value,
}
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Raw6Lease {
    #[serde(rename = "ip-address")]
    address: Ipv6Addr,
    #[serde(default, rename = "hw-address")]
    hwaddr: Option<String>, // MacAddr
    #[serde(rename = "valid-lft")]
    valid_lifetime: u64,
    #[serde(rename = "type")]
    _type: String,
    #[serde(default)]
    hostname: Option<String>,
    state: u64,
    #[serde(rename = "subnet-id")]
    subnet_id: u64,
    #[serde(flatten)]
    other: serde_json::Value,
}

impl Into<Lease> for Raw6Lease {
    fn into(self) -> Lease {
        (&self).into()
    }
}
impl Into<Lease> for &Raw6Lease {
    fn into(self) -> Lease {
        let mut hostname: String = "default".to_owned();
        if let Some(val) = &self.hostname {
            if let Some(val) = val.strip_suffix(".") {
                hostname = val.to_owned();
            }
            hostname = hostname.to_owned();
        }
        let macaddr: String = if let Some(hwaddr) = &self.hwaddr {
            hwaddr.to_owned()
        } else {
            MacAddr6::nil().to_string()
        };
        Lease {
            address: IpAddr::V6(self.address),
            hostname,
            mac: MacAddr6::from_str(&macaddr).unwrap(),
        }
    }
}
impl Into<Lease> for Raw4Lease {
    fn into(self) -> Lease {
        (&self).into()
    }
}
impl Into<Lease> for &Raw4Lease {
    fn into(self) -> Lease {
        let mut hostname: String = "default".to_owned();
        if let Some(val) = &self.hostname {
            if let Some(val) = val.strip_suffix(".") {
                hostname = val.to_owned();
            }
            hostname = hostname.to_owned();
        }
        let mut macaddr: String = MacAddr6::nil().to_string();
        if let Some(hwaddr) = &self.hwaddr {
            if !hwaddr.is_empty() {
                macaddr = hwaddr.to_owned();
            }
        }
        Lease {
            address: IpAddr::V4(self.address),
            hostname,
            mac: MacAddr6::from_str(&macaddr).unwrap(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn get_dhcp_cli() -> Result<()> {
        let config = Config::get()?;
        let cli = KeaDhcp::new().config(&config).build().await?;
        Ok(())
    }
}
