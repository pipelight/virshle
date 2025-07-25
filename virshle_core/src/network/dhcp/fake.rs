use super::IpPool;
use ipnet::{
    IpAddrRange, IpNet, IpSub, IpSubnets, Ipv4AddrRange, Ipv6AddrRange, Ipv6Net, Ipv6Subnets,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use rand::prelude::*;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::str::FromStr;

use crate::config::VirshleConfig;

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::{prelude::*, *};
use sea_orm::ColumnTrait;
use sea_orm::{prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult};

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FakeDhcp {
    pub pool: HashMap<String, IpPool>,
}

impl FakeDhcp {
    pub async fn delete_leases(vm_id: i32) -> Result<(), VirshleError> {
        let db = connect_db().await?;
        database::prelude::Lease::delete_many()
            .filter(database::entity::lease::Column::VmId.eq(vm_id))
            .exec(&db)
            .await?;
        Ok(())
    }
}

pub struct FakeDhcpLease {
    id: u64,
    vm_id: u64,
}
pub struct LeaseList {
    vm_id: u64,
    ip: Vec<IpNet>,
}

impl IpPool {
    pub fn get_mask(&self) -> Result<IpAddr, VirshleError> {
        let default_mask = IpAddr::V6(Ipv6Addr::from_str("ffff:ffff:ffff:ffff::").unwrap());
        Ok(default_mask)
    }
    pub async fn is_ip_already_leased(ip_addr: &IpAddr) -> Result<bool, VirshleError> {
        let db = connect_db().await?;
        let lease = database::lease::Entity::find()
            .filter(database::entity::lease::Column::Ip.eq(ip_addr.to_string()))
            .one(&db)
            .await?;
        Ok(lease.is_some())
    }
    /*
     * Get a random an unused ip.
     */
    pub async fn get_random_unleased_ip(&self) -> Result<IpAddr, VirshleError> {
        // Parse range from configuration
        let range: IpAddrRange;
        match self.subnet {
            IpNet::V6(subnet) => {
                match self.range {
                    [IpAddr::V6(start), IpAddr::V6(end)] => {
                        range = IpAddrRange::from(Ipv6AddrRange::new(start, end));
                    }
                    _ => {
                        return Err(LibError::builder()
                            .msg("Bad pool configuration.")
                            .help("")
                            .build()
                            .into());
                    }
                };
            }
            IpNet::V4(subnet) => {
                match self.range {
                    [IpAddr::V4(start), IpAddr::V4(end)] => {
                        range = IpAddrRange::from(Ipv4AddrRange::new(start, end));
                    }
                    _ => {
                        return Err(LibError::builder()
                            .msg("Bad pool configuration.")
                            .help("")
                            .build()
                            .into());
                    }
                };
            }
        };

        // Get random ip from range
        match range.choose(&mut rand::rng()) {
            Some(i) => {
                if futures::executor::block_on(Self::is_ip_already_leased(&i))? {
                    return Err(LibError::builder()
                        .msg("Thir random ip has already been leased.")
                        .help("You are cursed. Rerun the command to have more luck.")
                        .build()
                        .into());
                } else {
                    return Ok(i);
                }
            }
            None => {
                return Err(LibError::builder()
                    .msg("Couldn't get a random ip.")
                    .help("shit!")
                    .build()
                    .into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Result;

    #[tokio::test]
    async fn test_get_random_ip() -> Result<()> {
        let pool = IpPool {
            subnet: IpNet::V6(Ipv6Net::from_str("2001:db8::/64").into_diagnostic()?),
            range: [
                IpAddr::V6(Ipv6Addr::from_str("2001:db8::1ff").into_diagnostic()?),
                IpAddr::V6(Ipv6Addr::from_str("2001:db8::ffff").into_diagnostic()?),
            ],
        };
        let ip = pool.get_random_unleased_ip().await?;
        println!("{:#?}", ip);
        Ok(())
    }
}
