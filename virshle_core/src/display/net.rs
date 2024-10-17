use super::default;

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fmt;
use tabled::{
    settings::{object::Columns, Disable, Style},
    Table, Tabled,
};
use uuid::Uuid;

// Error Handling
use crate::cloud_hypervisor::{LinkState, Net};
use log::{log_enabled, Level};
use miette::{IntoDiagnostic, Result};
use virshle_error::VirshleError;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub struct NetTable {
    pub name: String,
    #[tabled(display_with = "display_state")]
    pub state: LinkState,
    pub uuid: Uuid,
}

impl NetTable {
    async fn from(e: &Net) -> Result<Self, VirshleError> {
        let table = NetTable {
            name: e.name.to_owned(),
            state: e.get_state()?,
            uuid: e.uuid,
        };
        Ok(table)
    }
}

pub fn display_state(state: &LinkState) -> String {
    let res = match state {
        LinkState::Up => "up".green().to_string(),
        LinkState::Down => "down".red().to_string(),
        LinkState::NotCreated => "not_created".white().to_string(),
    };
    format!("{}", res)
}

impl NetTable {
    pub async fn display(items: Vec<Self>) -> Result<()> {
        if log_enabled!(Level::Info) {
            let mut res = Table::new(&items);
            res.with(Style::rounded());
            println!("{}", res);
        } else {
            let mut res = Table::new(&items);
            res.with(Disable::column(Columns::last()));
            res.with(Style::rounded());
            println!("{}", res);
        }
        Ok(())
    }
}
impl Net {
    pub async fn display(items: Vec<Self>) -> Result<()> {
        let mut table: Vec<NetTable> = vec![];
        for e in items {
            table.push(NetTable::from(&e).await?);
        }
        NetTable::display(table).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_display_state() -> Result<()> {
        println!("\n{}", display_state(&LinkState::Down));
        Ok(())
    }
    #[test]
    fn display_mock() -> Result<()> {
        let items = vec![
            NetTable {
                uuid: Uuid::new_v4(),
                name: "net_arch".to_owned(),
                state: LinkState::Up,
            },
            NetTable {
                uuid: Uuid::new_v4(),
                name: "net_nix".to_owned(),
                state: LinkState::Down,
            },
        ];

        println!("");
        default(items)?;

        Ok(())
    }
}
