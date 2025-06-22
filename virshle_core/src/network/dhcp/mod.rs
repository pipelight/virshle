use ipnet::{IpAddrRange, IpNet, IpSub, IpSubnets, Ipv4AddrRange, Ipv6AddrRange, Ipv6Subnets};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FakeDhcp {
    pool: Vec<IpPool>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IpPool {
    subnet: IpNet,
    range: [IpAddr; 2],
}
impl IpPool {
    /*
     * Get a random an unused ip.
     */
    pub fn get_random_ip(&self) -> Result<IpAddr, VirshleError> {
        // Parse range from configuration
        let range: IpAddrRange;
        match self.subnet {
            IpNet::V6(subnet) => {
                match self.range {
                    [IpAddr::V6(start), IpAddr::V6(end)] => {
                        range = IpAddrRange::from(Ipv6AddrRange::new(start, end));
                    }
                    _ => {}
                };
            }
            IpNet::V4(subnet) => {
                match self.range {
                    [IpAddr::V4(start), IpAddr::V4(end)] => {
                        range = IpAddrRange::from(Ipv4AddrRange::new(start, end));
                    }
                    _ => {}
                };
            }
        };

        // Get random ip from range
        match range.choose(&mut rand::rng()) {
            Some(i) => {}
            None => {}
        };
    }
}

pub trait Leases {
    fn get_all() -> Result<(), VirshleError>;
}

pub struct FakeDhcpLease {
    id: u64,
    vm_id: u64,
}
pub struct LeaseList {
    vm_id: u64,
    ip: Vec<IpNet>,
}

pub fn get_all() -> Result<(), VirshleError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Result;

    #[test]
    fn test_get_random_ip() -> Result<()> {
        Ok(())
    }
}
