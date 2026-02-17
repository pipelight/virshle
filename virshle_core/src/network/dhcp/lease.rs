// IP
use serde::{Deserialize, Serialize};

// Net primitives
use macaddr::MacAddr6;
use std::net::IpAddr;

// Error handling
use miette::Result;
use virshle_error::VirshleError;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Lease {
    pub address: IpAddr,
    pub hostname: String,
    pub mac: MacAddr6,
}

pub fn get_all() -> Result<(), VirshleError> {
    Ok(())
}
