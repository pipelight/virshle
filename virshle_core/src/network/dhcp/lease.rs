// IP
use ipnet::{
    IpAddrRange, IpNet, IpSub, IpSubnets, Ipv4AddrRange, Ipv6AddrRange, Ipv6Net, Ipv6Subnets,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub trait Leases {
    fn get_all() -> Result<(), VirshleError>;
}

pub fn get_all() -> Result<(), VirshleError> {
    Ok(())
}
