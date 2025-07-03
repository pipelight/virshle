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
use std::collections::HashMap;

// Error handling
use log::trace;
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeaDhcp {
    pub pool: Option<HashMap<String, IpPool>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Raw6Lease {
    address: IpAddr,
    duid: String,
    valid_lifetime: u64,
    expire: u64,
    subnet_id: u64,
    pref_lifetime: u64,
    lease_type: u64,
    iaid: String,
    prefix_len: u64,
    fqdn_fwd: u64,
    fqdn_rev: u64,
    hostname: String,
    hwaddr: String, // MacAddr
    state: u64,
    user_context: String,
    hwtype: u64,
    hwaddr_source: u64,
    pool_id: u64,
}
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Raw4Lease {
    address: IpAddr,
    hwaddr: String, // MacAddr
    client_id: String,
    valid_lifetime: u64,
    expire: u64,
    subnet_id: u64,
    fqdn_fwd: u64,
    fqdn_rev: u64,
    hostname: String,
    state: u64,
    user_context: String,
    pool_id: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Lease {
    pub address: IpAddr,
    pub hostname: String,
    hwaddr: String,
}
impl From<&Raw6Lease> for Lease {
    fn from(e: &Raw6Lease) -> Self {
        Lease {
            address: e.address,
            hostname: e.hostname.clone(),
            hwaddr: e.hwaddr.clone(), // hwaddr: MacAddr::V6(MacAddr6::from_str(&e.hwaddr)),
        }
    }
}
impl From<&Raw4Lease> for Lease {
    fn from(e: &Raw4Lease) -> Self {
        Lease {
            address: e.address,
            hostname: e.hostname.clone(),
            hwaddr: e.hwaddr.clone(), // hwaddr: MacAddr::V6(MacAddr6::from_str(&e.hwaddr)),
        }
    }
}

pub const LEASES_DIR: &'static str = "/var/lib/kea";

impl KeaDhcp {
    pub fn get_leases_by_hostname(hostname: &str) -> Result<Vec<Lease>, VirshleError> {

        let leases6: Vec<Lease> = Self::get_ipv6_leases()?
            .into_iter()
            .filter(|e| 
                // kea ipv6 hostname
                e.hostname.strip_suffix(".") == Some(hostname)
            )
            .collect();
        
        let leases4: Vec<Lease> = Self::get_ipv4_leases()?
            .into_iter()
            .filter(|e| 
                    e.hostname == hostname
            )
            .collect();

        // Get ipv4 by mac address.
        let leases4_w_hwaddr: Vec<Lease> = Self::get_ipv4_leases()?
            .into_iter()
            .filter(|e| 
                if let Some(lease6) = leases6.last() {
                    e.hwaddr == lease6.hwaddr
                }
                else {
                    false
                }
            )
            .collect();

        let mut leases: Vec<Lease> = vec![];
        leases.extend(leases6);
        leases.extend(leases4);
        leases.extend(leases4_w_hwaddr);
        leases.dedup();

        Ok(leases)
    }

    pub fn get_leases() -> Result<Vec<Lease>, VirshleError> {
        let mut leases: Vec<Lease> = Self::get_ipv6_leases()?;
        leases.extend(Self::get_ipv4_leases()?);

        Ok(leases)
    }

    pub fn get_ipv4_leases() -> Result<Vec<Lease>, VirshleError> {
        let path = Path::new(&LEASES_DIR);
        let mut leases = vec![];
        for entry in path.read_dir()? {
            if let Ok(entry) = entry {
                if entry.path().is_file()
                    && entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .starts_with("dhcp4.leases")
                {
                    trace!("reading csv file: {}", entry.path().to_str().unwrap());
                    let mut reader = csv::Reader::from_path(entry.path())?;
                    for result in reader.deserialize() {
                        let record: Raw4Lease = result?;
                        let lease = Lease::from(&record);
                        leases.push(lease);
                    }
                }
            }
        }
        leases.dedup();
        Ok(leases)
    }

    pub fn get_ipv6_leases() -> Result<Vec<Lease>, VirshleError> {
        let path = Path::new(&LEASES_DIR);
        let mut leases = vec![];
        for entry in path.read_dir()? {
            if let Ok(entry) = entry {
                if entry.path().is_file()
                    && entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .starts_with("dhcp6.leases")
                {
                    trace!("reading csv file: {}", entry.path().to_str().unwrap());
                    let mut reader = csv::Reader::from_path(entry.path())?;
                    for result in reader.deserialize() {
                        let record: Raw6Lease = result?;
                        let lease = Lease::from(&record);
                        leases.push(lease);
                    }
                }
            }
        }
        leases.dedup();
        Ok(leases)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_leases() -> Result<()> {
        let res = KeaDhcp::get_leases()?;
        println!("{:#?}", res);
        Ok(())
    }
}
