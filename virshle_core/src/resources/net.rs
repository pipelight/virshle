use serde::{Deserialize, Serialize};
use tabled::Tabled;
use uuid::Uuid;

// Error Handling
use crate::error::{VirshleError, WrapError};
use log::trace;
use miette::{IntoDiagnostic, Result};

// libvirt
use super::connect;
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
        let network = Network::lookup_by_name(&conn, name)?;

        let item = Net {
            uuid: network.get_uuid()?,
            name: network.get_name()?,
            state: State::from(network.is_active()?),
            ..Default::default()
        };
        Ok(item)
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fetch_networks() -> Result<()> {
        let items = Net::get_all();
        println!("{:#?}", items);
        Ok(())
    }
}
