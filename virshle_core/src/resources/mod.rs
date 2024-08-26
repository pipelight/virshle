pub mod net;
pub mod vm;
use crossterm::{execute, style::Stylize, terminal::size};
use owo_colors::OwoColorize;

use crate::convert;
use log::{info, log_enabled, Level};

use bat::PrettyPrinter;
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

pub fn create_resources(toml: &str) -> Result<()> {
    let (cols, rows) = size().into_diagnostic()?;
    let divider = "-".repeat((cols / 3).into());

    if log_enabled!(Level::Info) {
        println!("{}", format!("{divider}toml{divider}").green());
        PrettyPrinter::new()
            .input_from_bytes(toml.as_bytes())
            .language("toml")
            .print()
            .into_diagnostic()?;
        println!("");
    }

    let value = convert::from_toml(&toml)?;
    if let Some(map) = value.as_object() {
        for key in map.keys() {
            let mut new_map = Map::new();
            new_map.insert(key.to_owned(), map.get(key).unwrap().to_owned());
            let xml = convert::to_xml(&Value::Object(new_map))?;

            if log_enabled!(Level::Info) {
                println!("{}", format!("{divider}xml{divider}").green());
                PrettyPrinter::new()
                    .input_from_bytes(xml.as_bytes())
                    .language("xml")
                    .print()
                    .into_diagnostic()?;
                println!("");
            }

            if key == "domain" {
                Vm::set_xml(&xml)?;
            }
            if key == "network" {
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
