// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub trait Leases {
    fn get_all() -> Result<(), VirshleError>;
}

pub struct DnsMasq;
impl Leases for DnsMasq {
    fn get_all() -> Result<(), VirshleError> {
        let filepath = "/tmp/";
        Ok(())
    }
}

pub struct DhcpLease {
    linkId: u64,
    mac: String,
    hostname: String,
}
pub struct LeaseList {
    mac: String,
    hostname: String,
    ipv6: Vec<String>,
}

pub fn get_all() -> Result<(), VirshleError> {
    Ok(())
}
