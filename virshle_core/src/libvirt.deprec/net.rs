use bon::{bon, Builder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, u32};
use tabled::Tabled;
use uuid::Uuid;

// Error Handling
use crate::error::{VirshleError, VirtError, WrapError};
use log::trace;
use miette::{IntoDiagnostic, Result};
use std::collections::HashMap;

// libvirt
use super::connect;
use crate::convert;
use convert_case::{Case, Casing};
use strum::{EnumIter, IntoEnumIterator};
use virt::network::Network;

use once_cell::sync::Lazy;

static NVirConnectListAllNetworksFlags: u32 = 5;

fn display_option(state: &Option<State>) -> String {
    match state {
        Some(s) => format!("{}", s),
        None => format!(""),
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq, EnumIter)]
pub enum State {
    #[default]
    Inactive = 0,
    Active = 1,
}
impl From<u32> for State {
    fn from(value: u32) -> Self {
        match value {
            0 => State::Inactive,
            1 => State::Active,
            _ => State::Inactive,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub struct Net {
    pub uuid: Uuid,
    pub name: String,
    pub state: State,
    pub autostart: bool,
    pub persistent: bool,
}

// Methods for direct interactions with libvirt api.
impl Net {
    // Setters
    pub fn from_path(toml: &str) -> Result<Self, VirshleError> {
        let toml = fs::read_to_string(toml)?;
        let xml = convert::from_toml_to_xml(&toml)?;
        let res = Self::set_xml(&xml)?;
        Ok(res)
    }
    fn from_string(toml: &str) -> Result<Self, VirshleError> {
        let xml = convert::from_toml_to_xml(&toml)?;
        let res = Self::set_xml(&xml)?;
        Ok(res)
    }
    fn from_value(value: &serde_json::Value) -> Result<Self, VirshleError> {
        let xml = convert::to_xml(value)?;
        let res = Self::set_xml(&xml)?;
        Ok(res)
    }
    /*
     * Set a network definition from Xml.
     */
    pub fn set_xml(xml: &str) -> Result<Net, VirshleError> {
        let conn = connect()?;
        let network = Network::define_xml(&conn, &xml);
        match network {
            Ok(res) => Ok(Self::from(&res)?),
            Err(e) => Err(VirtError::new("The network could not be created.", "", e).into()),
        }
    }
    /*
     * Set a network definition from Xml.
     * And silently fail.
     */
    pub fn ensure_xml(xml: &str) -> Result<(), VirshleError> {
        let res = Self::set_xml(xml);
        match res {
            Ok(_) => {}
            Err(e) => {}
        };
        Ok(())
    }

    // Getters
    pub fn get_by_name(name: &str) -> Result<Self, VirshleError> {
        let conn = connect()?;
        let res = Network::lookup_by_name(&conn, name);
        match res {
            Ok(e) => {
                let item = Net::from(&e)?;
                Ok(item)
            }
            Err(e) => Err(VirtError::new(
                &format!("No network with name {:?}", name),
                "Maybe you made a typo",
                e,
            )
            .into()),
        }
    }
    pub fn get_all() -> Result<Vec<Self>, VirshleError> {
        let conn = connect()?;
        let mut map: HashMap<String, Net> = HashMap::new();
        for flag in State::iter() {
            let items = conn.list_all_networks(flag as u32)?;
            for e in items.clone() {
                let network = Net::from(&e)?;
                let name = network.clone().name;
                if !map.contains_key(&name) {
                    map.insert(name, network);
                }
            }
        }
        let mut list: Vec<Net> = map.into_values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(list)
    }

    // Convert Libvirt struct
    fn from(e: &Network) -> Result<Net, VirshleError> {
        let res = Net {
            uuid: e.get_uuid()?,
            name: e.get_name()?,
            state: State::from(e.is_active()? as u32),
            autostart: e.get_autostart()?,
            persistent: e.is_persistent()?,
        };
        Ok(res)
    }
}
impl Net {
    pub fn definition(&self) -> Result<Value, VirshleError> {
        let conn = connect()?;
        let network = Network::lookup_by_name(&conn, &self.name)?;
        let xml = network.get_xml_desc(1)?;
        println!("{}", xml);
        let value = convert::from_xml(&xml)?;
        Ok(value)
    }

    pub fn delete(&self) -> Result<(), VirshleError> {
        let conn = connect()?;
        let item = Network::lookup_by_name(&conn, &self.name)?;

        let res = item.destroy();
        match res {
            Ok(res) => Ok(()),
            Err(e) => Err(VirtError::new("The network could not be destroyed", "", e).into()),
        }
    }
    pub fn start(&self) -> Result<(), VirshleError> {
        let conn = connect()?;
        let net = Network::lookup_by_name(&conn, &self.name)?;

        let res = net.create();
        match res {
            Ok(res) => Ok(()),
            Err(e) => Err(VirtError::new("The network could not be started", "", e).into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn fetch_networks() -> Result<()> {
        let items = Net::get_all();
        println!("{:#?}", items);
        Ok(())
    }

    // #[test]
    fn create_network() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../templates/net/default.toml");
        let path = path.display().to_string();

        Net::from_path(&path)?;
        Ok(())
    }

    #[test]
    fn get_network_definition() -> Result<()> {
        let def = Net::get_by_name("default_6")?.definition()?;
        println!("{:#?}", def);
        Ok(())
    }

    // #[test]
    fn delete_network() -> Result<()> {
        Net::get_by_name("default_6")?.delete()?;
        Ok(())
    }
}
