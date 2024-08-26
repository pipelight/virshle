use serde::{Deserialize, Serialize};
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

static NVirConnectListAllDomainsFlags: u32 = 15;

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
    #[tabled(skip)]
    pub uuid: Uuid,
    pub name: String,
    // #[tabled(display_with = "display_option")]
    pub state: State,
    pub autostart: bool,
    pub persistent: bool,
}
impl Net {
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
    pub fn get(name: &str) -> Result<Self, VirshleError> {
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
        let list: Vec<Net> = map.into_values().collect();
        Ok(list)
    }
    pub fn set(path: &str) -> Result<(), VirshleError> {
        let toml = fs::read_to_string(path)?;
        let xml = convert::from_toml_to_xml(&toml)?;
        Self::set_xml(&xml)?;
        Ok(())
    }
    pub fn set_xml(xml: &str) -> Result<(), VirshleError> {
        let conn = connect()?;
        let res = Network::create_xml(&conn, &xml);
        match res {
            Ok(res) => Ok(()),
            Err(e) => Err(VirtError::new(
                "The network could not be created",
                "Try deleting the network first",
                e,
            )
            .into()),
        }
    }
    pub fn delete(name: &str) -> Result<(), VirshleError> {
        // Guard
        Self::get(name)?;

        let conn = connect()?;
        let item = Network::lookup_by_name(&conn, name)?;
        let res = item.destroy();
        match res {
            Ok(res) => Ok(()),
            Err(e) => Err(VirtError::new("The network could not be destroyed", "", e).into()),
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

    #[test]
    fn create_network() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../templates/network/base.toml");
        let path = path.display().to_string();

        Net::set(&path)?;

        Ok(())
    }

    #[test]
    fn delete_network() -> Result<()> {
        Net::delete("default_6")?;

        Ok(())
    }
}
