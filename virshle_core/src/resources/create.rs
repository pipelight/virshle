use crossterm::{execute, style::Stylize, terminal::size};
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::str::FromStr;
use strum::{Display, EnumIter, EnumString, FromRepr, IntoEnumIterator};
use uuid::Uuid;

use crate::convert;
use serde_json::{Map, Value};

use super::{net::Net, secret::Secret, vm::Vm};

// Error Handling
use crate::error::{VirshleError, WrapError};
use log::{info, log_enabled, Level};
use miette::{IntoDiagnostic, Result};

#[derive(Debug, EnumIter, EnumString, Display)]
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

pub fn create(toml: &str) -> Result<(), VirshleError> {
    let mut value = convert::from_toml(&toml)?;

    if let Some(map) = value.as_object_mut() {
        let mut resources: HashMap<String, Vec<Value>> = HashMap::new();
        resources.insert(ResourceType::Secret.to_string(), vec![]);
        resources.insert(ResourceType::Net.to_string(), vec![]);
        resources.insert(ResourceType::Vm.to_string(), vec![]);

        // Reorder values for efficient creation.
        let keys: Vec<String> = map.keys().map(|e| e.to_owned()).collect();
        for key in keys {
            let binding = ResourceType::from_str(&key)?;
            match binding {
                ResourceType::Secret => {
                    let mut new_map = Map::new();
                    new_map.insert(key.to_owned(), map.get(&key).unwrap().to_owned());
                    let value = Value::Object(new_map);

                    resources
                        .get_mut(&ResourceType::Secret.to_string())
                        .unwrap()
                        .push(value);
                }
                ResourceType::Net => {
                    let mut new_map = Map::new();
                    new_map.insert(key.to_owned(), map.get(&key).unwrap().to_owned());
                    let value = Value::Object(new_map);

                    resources
                        .get_mut(&ResourceType::Net.to_string())
                        .unwrap()
                        .push(value);
                }
                ResourceType::Vm => {
                    let mut new_map = Map::new();
                    let mut definition = map.get_mut(&key).unwrap().to_owned();

                    // Set Vm uniq ids
                    let new_uuid = Uuid::new_v4().to_string();
                    let new_name = convert::random_name()?;

                    let mutable = definition.as_object_mut().unwrap();
                    if let Some(name) = mutable.get_mut("name") {
                        *name = Value::String(new_name);
                    }
                    if let Some(uuid) = mutable.get_mut("uuid") {
                        *uuid = Value::String(new_uuid.clone());
                    }
                    convert::relpath_to_copy(&mut definition, &new_uuid)?;

                    new_map.insert(key.to_owned(), definition);
                    let value = Value::Object(new_map);

                    resources
                        .get_mut(&ResourceType::Vm.to_string())
                        .unwrap()
                        .push(value);
                }
                ResourceType::Vol => {}
                _ => {}
            };
        }
        for key in resources.keys() {
            let values = resources.get(key).unwrap();
            let binding = ResourceType::from_str(key)?;
            match binding {
                ResourceType::Secret => {
                    for v in values {
                        Secret::set_multi_xml_w_value(v)?;
                    }
                }
                ResourceType::Net => {
                    for v in values {
                        let xml = convert::to_xml(&v)?;
                        Net::ensure_xml(&xml)?;
                    }
                }
                ResourceType::Vm => {
                    for v in values {
                        let xml = convert::to_xml(&v)?;
                        Vm::set_xml(&xml)?;
                    }
                }
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
