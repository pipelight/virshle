pub mod net;
pub mod vm;

pub use net::Net;
pub use vm::Vm;

// Error Handling
use crate::error::{VirshleError, WrapError};
use log::trace;
use miette::{IntoDiagnostic, Result};

use virt::connect::Connect;

pub fn connect() -> Result<Connect, VirshleError> {
    // let conn = Connect::open(Some("test:///default")).into_diagnostic()?;
    let res = Connect::open(Some("qemu:///system"))?;
    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_connect() -> Result<()> {
        let res = connect();
        assert!(res.is_ok());
        Ok(())
    }
}
