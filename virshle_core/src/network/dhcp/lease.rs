// IP
use ipnet::{
    IpAddrRange, IpNet, IpSub, IpSubnets, Ipv4AddrRange, Ipv6AddrRange, Ipv6Net, Ipv6Subnets,
};
use serde::{Deserialize, Serialize};

// Net primitives
use macaddr::{MacAddr, MacAddr6};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Lease {
    pub address: IpAddr,
    pub hostname: String,
    pub mac: MacAddr6,
}

pub fn get_all() -> Result<(), VirshleError> {
    Ok(())
}
