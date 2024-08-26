use serde::{Deserialize, Serialize};
use std::fs;
use tabled::Tabled;
use uuid::Uuid;

// Error Handling
use crate::error::{VirshleError, VirtError, WrapError};
use log::trace;
use miette::{IntoDiagnostic, Result};

// libvirt
use super::connect;
use crate::convert;
use convert_case::{Case, Casing};
use strum::EnumIter;
use virt::network::Network;

fn display_option(state: &Option<State>) -> String {
    match state {
        Some(s) => format!("{}", s),
        None => format!(""),
    }
}
#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub struct Net {
    #[tabled(order = 2)]
    pub uuid: Uuid,
    pub name: String,
    // #[tabled(display_with = "display_option")]
    pub state: State,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq, EnumIter)]
pub enum State {
    #[default]
    Inactive,
    Active,
}
impl From<bool> for State {
    fn from(value: bool) -> Self {
        match value {
            true => State::Active,
            false => State::Inactive,
        }
    }
}
impl Net {
    pub fn get(name: &str) -> Result<Self, VirshleError> {
        let conn = connect()?;
        let res = Network::lookup_by_name(&conn, name);
        match res {
            Ok(network) => {
                let item = Net {
                    uuid: network.get_uuid()?,
                    name: network.get_name()?,
                    state: State::from(network.is_active()?),
                    ..Default::default()
                };
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
    pub fn get_all() -> Result<Vec<Self>> {
        let conn = connect()?;
        let names = conn.list_networks().into_diagnostic()?;
        let mut list = vec![];
        for name in names {
            list.push(Net::get(&name)?);
        }
        Ok(list)
    }
    pub fn delete(name: &str) -> Result<(), VirshleError> {
        // Guard
        Self::get(name)?;

        let conn = connect()?;
        let network = Network::lookup_by_name(&conn, name)?;
        let res = network.destroy();
        match res {
            Ok(res) => Ok(()),
            Err(e) => Err(VirtError::new("The network could not be destroyed", "", e).into()),
        }
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
