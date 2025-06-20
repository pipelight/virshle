use ipnet::IpNet;

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

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
