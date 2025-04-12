use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, Value::Array};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

// Network primitives
use macaddr::MacAddr8;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
// Error handling
use log::info;
use miette::{IntoDiagnostic, Result};
use pipelight_exec::Process;
use virshle_error::{LibError, VirshleError, WrapError};

use super::InterfaceState;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct OvsInterface {
    #[serde(rename = "_uuid")]
    uuid: Uuid,
    #[serde(rename = "ifindex")]
    pub index: Option<u64>,
    pub name: String,
    #[serde(rename = "mac_in_use")]
    pub mac: String,
    #[serde(rename = "admin_state")]
    pub state: String,
}
#[derive(Default, Clone, Debug)]
pub struct Ovs;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct OvsResponse {
    data: Vec<Vec<Value>>,
    headings: Vec<String>,
}

// pub struct OvsResponse {
//     bridges: Vec<Bridge>,
// }

pub enum OvsValue {
    String,
    Map,
    Set,
    Bool,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Bridge {
    #[serde(rename = "_uuid")]
    uuid: Uuid,
    name: String,
    ports: Vec<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Port {
    uuid: Uuid,
    name: String,
    // interfaces: Vec<Interface>
}

impl Ovs {
    /*
     * Split host main network interface to provide connectivity to vms.
     * see: ./README.md
     */
    pub fn make_host_switch() -> Result<(), VirshleError> {
        // Find main interface
        Ok(())
    }

    /*
     * Convert ovs json types into sane and readable json.
     * Recursive
     */
    pub fn bad_json_to_good_json(value: &Value) -> Result<Value, VirshleError> {
        if let Some(array) = value.as_array() {
            let mut array = array.clone();
            let data_type = array.remove(0);
            let data_type = data_type.as_str().unwrap();

            let data = array.remove(0).to_owned();

            return match data_type {
                // Identifier
                "uuid" => Ok(data.to_owned()),
                // Array or Vec
                "set" => {
                    let data = data.as_array().unwrap();

                    let mut new_value = vec![];
                    for item in data {
                        let mut item = item.as_array().unwrap().to_vec();
                        if !item.is_empty() {
                            let heading = item.remove(0);
                            let heading = heading.as_str().unwrap().to_owned();
                            let data = item.remove(0);
                            new_value.push(data.to_owned());
                        }
                    }
                    return Ok(Value::Array(new_value));
                }
                // Object
                "map" => {
                    // for item in array
                    let data = data.as_array().unwrap();

                    let mut new_value = Map::new();
                    for item in data {
                        let mut item = item.as_array().unwrap().to_vec();
                        if !item.is_empty() {
                            let heading = item.remove(0);
                            let heading = heading.as_str().unwrap().to_owned();
                            let data = item.remove(0);
                            new_value.insert(heading, data.to_owned());
                        }
                    }
                    return Ok(Value::Object(new_value));
                }
                _ => {
                    return Ok(Value::Null);
                }
            };
        } else if value.is_string() || value.is_boolean() || value.is_number() {
            return Ok(value.to_owned());
        } else {
            return Ok(Value::Null);
        }
    }
    /*
     * Convert ovs-vsctl json to better json
     */
    pub fn to_json(response: &str) -> Result<Value, VirshleError> {
        let ovs_reponse: OvsResponse = serde_json::from_str(&response)?;

        // Iterate response elements
        let mut items: Vec<Value> = vec![];
        for item in ovs_reponse.data {
            let mut kv = Map::new();
            for (key, value) in ovs_reponse.headings.iter().zip(item) {
                kv.insert(key.to_owned(), Self::bad_json_to_good_json(&value)?);
            }
            items.push(Value::Object(kv.clone()));
        }
        let value = Value::Array(items.clone());
        Ok(value)
    }
    pub fn get_bridges(&self) -> Result<Vec<Bridge>, VirshleError> {
        let cmd = "sudo ovs-vsctl -f json list bridge".to_owned();

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        let mut bridges = vec![];
        if let Some(stdout) = res.io.stdout {
            bridges = serde_json::from_value(Self::to_json(&stdout)?)?;
        }
        Ok(bridges)
    }
    pub fn get_interfaces(&self) -> Result<Vec<OvsInterface>, VirshleError> {
        let cmd = "sudo ovs-vsctl -f json list interface".to_owned();

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        let mut bridges = vec![];
        if let Some(stdout) = res.io.stdout {
            bridges = serde_json::from_value(Self::to_json(&stdout)?)?;
        }
        Ok(bridges)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Brigdges
    // #[test]
    fn test_bridges_to_json() -> Result<()> {
        let cmd = "sudo ovs-vsctl -f json list bridge".to_owned();
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        if let Some(stdout) = res.io.stdout {
            let res = Ovs::to_json(&stdout)?;
            println!("{:#?}", res);
        }

        Ok(())
    }
    // #[test]
    fn test_ovs_get_bridges() -> Result<()> {
        let ovs = Ovs::default();
        let res = ovs.get_bridges();
        println!("{:#?}", res);
        Ok(())
    }

    // Interfaces
    #[test]
    fn test_interfaces_to_json() -> Result<()> {
        let cmd = "sudo ovs-vsctl -f json list interface".to_owned();
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        if let Some(stdout) = res.io.stdout {
            let res = Ovs::to_json(&stdout)?;
            println!("{:#?}", res);
        }

        Ok(())
    }
    #[test]
    fn test_ovs_get_interfaces() -> Result<()> {
        let ovs = Ovs::default();
        let res = ovs.get_interfaces();
        println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn test_ovs_make_patch() -> Result<()> {
        let ovs = Ovs::default();
        // ovs.make_host_switch()?;
        Ok(())
    }
}
