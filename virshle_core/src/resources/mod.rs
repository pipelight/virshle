pub mod net;
pub mod secret;
pub mod vm;
use crossterm::{execute, style::Stylize, terminal::size};
use owo_colors::OwoColorize;
use std::str::FromStr;
use strum::{Display, EnumIter, EnumString, FromRepr, IntoEnumIterator};

use crate::convert;
use serde_json::{Map, Value};

use bat::PrettyPrinter;
pub use net::Net;
pub use secret::Secret;
pub use vm::Vm;

// Error Handling
use crate::error::{VirshleError, WrapError};
use log::{info, log_enabled, Level};
use miette::{IntoDiagnostic, Result};

use virt::connect::Connect;

pub fn connect() -> Result<Connect, VirshleError> {
    // let conn = Connect::open(Some("test:///default")).into_diagnostic()?;
    let res = Connect::open(Some("qemu:///system"))?;
    Ok(res)
}
#[derive(Debug, EnumIter, EnumString)]
pub enum ResourceType {
    #[strum(serialize = "network")]
    Net,
    #[strum(serialize = "domain")]
    Vm,
    #[strum(serialize = "secret")]
    Secret,
    #[strum(serialize = "volume")]
    Vol,
}

pub fn create_resources(toml: &str) -> Result<(), VirshleError> {
    let value = convert::from_toml(&toml)?;
    if let Some(map) = value.as_object() {
        for key in map.keys() {
            let binding = ResourceType::from_str(key)?;
            match binding {
                ResourceType::Vm => {
                    let mut new_map = Map::new();
                    new_map.insert(key.to_owned(), map.get(key).unwrap().to_owned());
                    let xml = convert::to_xml(&Value::Object(new_map))?;
                    Vm::set_xml(&xml)?;
                }
                ResourceType::Net => {
                    let mut new_map = Map::new();
                    new_map.insert(key.to_owned(), map.get(key).unwrap().to_owned());
                    let xml = convert::to_xml(&Value::Object(new_map))?;
                    Net::set_xml(&xml)?;
                }
                ResourceType::Secret => {
                    let mut new_map = Map::new();
                    new_map.insert(key.to_owned(), map.get(key).unwrap().to_owned());
                    let mut value = Value::Object(new_map);
                    Secret::set_multi_xml_w_value(&mut value)?;
                }
                ResourceType::Vol => {}
                _ => {}
            };
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
