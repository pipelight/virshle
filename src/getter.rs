use serde_json::{json, Map, Value};
use std::fs;
// Error Handling
use log::trace;
use miette::{IntoDiagnostic, Result};

use virt::connect::Connect;
use virt::domain::Domain;

pub fn connect() -> Result<()> {
    // let conn = Connect::open(Some("test:///default")).into_diagnostic()?;
    let conn = Connect::open(Some("qemu:///system")).into_diagnostic()?;
    Ok(())
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

    #[test]
    fn fetch_node_info() -> Result<()> {
        let res = connect();
        assert!(res.is_ok());
        Ok(())
    }
}
