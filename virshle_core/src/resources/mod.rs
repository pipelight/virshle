pub mod net;
pub mod vm;

use crate::convert;

pub use net::Net;
use serde_json::{Map, Value};
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

pub enum ResourceType {
    Net(String),
    Vm(String),
}

pub fn create_resources(value: &Value) -> Result<()> {
    if let Some(map) = value.as_object() {
        for key in map.keys() {
            if key == "domain" {
                let mut new_map = Map::new();
                new_map.insert(key.to_owned(), map.get(key).unwrap().to_owned());
                let xml = convert::to_xml(&Value::Object(new_map))?;
                Vm::set_xml(&xml)?;
            }
            if key == "network" {
                let mut new_map = Map::new();
                new_map.insert(key.to_owned(), map.get(key).unwrap().to_owned());
                let xml = convert::to_xml(&Value::Object(new_map))?;
                Net::set_xml(&xml)?;
            }
        }
    }
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
}
