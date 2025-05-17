use crate::config::{Node, NodeInfo};
use crate::connection::{ConnectionState, Uri};

use super::utils::{display_id, display_ips, display_some_num, display_some_vram};
use crate::cloud_hypervisor::{Vm, VmState};

use human_bytes::human_bytes;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tabled::{
    settings::{disable::Remove, object::Columns, themes::BorderCorrection, Panel, Style},
    Table, Tabled,
};
use uuid::Uuid;

// Error Handling
use log::{log_enabled, Level};
use miette::{IntoDiagnostic, Result};
use virshle_error::VirshleError;

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub struct NodeTable {
    pub name: String,
    #[tabled(display = "display_state")]
    pub state: ConnectionState,
    #[tabled(display = "display_some_num")]
    pub vm: Option<u64>,
    #[tabled(display = "display_some_num")]
    pub cpu: Option<u64>,
    #[tabled(display = "display_some_vram")]
    pub ram: Option<u64>,
}

impl NodeTable {
    async fn from(e: &(Node, (ConnectionState, Option<NodeInfo>))) -> Result<Self, VirshleError> {
        let (node, (state, node_info)) = e;
        let table;
        if let Some(node_info) = node_info {
            table = NodeTable {
                name: node.name.to_owned(),
                cpu: Some(node_info.host_info.cpu.number),
                ram: Some(node_info.host_info.ram.total),
                vm: Some(node_info.virshle_info.num_vm),
                state: state.to_owned(),
            };
        } else {
            table = NodeTable {
                name: node.name.to_owned(),
                cpu: None,
                ram: None,
                vm: None,
                state: state.to_owned(),
            };
        }
        Ok(table)
    }
}
impl NodeTable {
    pub fn display(items: Vec<Self>) -> Result<(), VirshleError> {
        let mut res = Table::new(&items);
        if log_enabled!(Level::Info) {
            res.with(Style::rounded());
        } else if log_enabled!(Level::Warn) {
            res.with(Remove::column(Columns::last()));
            res.with(Remove::column(Columns::last()));
            res.with(Style::rounded());
        } else {
            res.with(Remove::column(Columns::last()));
            res.with(Remove::column(Columns::last()));
            res.with(Remove::column(Columns::last()));
            res.with(Style::rounded());
        }
        println!("{}", res);
        Ok(())
    }
}

impl Node {
    pub async fn display(
        items: HashMap<Node, (ConnectionState, Option<NodeInfo>)>,
    ) -> Result<(), VirshleError> {
        let mut table: Vec<NodeTable> = vec![];
        for item in items {
            let e = NodeTable::from(&item).await?;
            table.push(e);
        }
        NodeTable::display(table)?;
        Ok(())
    }
}

pub fn display_state(state: &ConnectionState) -> String {
    let icon = "●";
    let res = match state {
        // Success
        ConnectionState::DaemonUp => format!("{} Running", icon).green().to_string(),

        // Uninitialized
        ConnectionState::Down => format!("{} Down", icon).white().to_string(),

        // Warning: small error
        ConnectionState::SshAuthError => format!("{} SshAuthError", icon).yellow().to_string(),

        // Error
        ConnectionState::SocketNotFound => format!("{} SocketNotFound", icon).red().to_string(),
        ConnectionState::DaemonDown => format!("{} DaemonDown", icon).red().to_string(),
        // Unknown network reason.
        ConnectionState::Unreachable => format!("{} Unreachable", icon).red().to_string(),
    };
    format!("{}", res)
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn display_mock() -> Result<()> {
        // Get nodes
        let nodes = vec![
            NodeTable {
                name: "node_1".to_owned(),
                cpu: None,
                ram: None,
                state: ConnectionState::Down,
                vm: None,
            },
            NodeTable {
                name: "node_2".to_owned(),
                cpu: Some(16),
                ram: Some(30 * u64::pow(1024, 4)),
                state: ConnectionState::DaemonUp,
                vm: Some(2),
            },
        ];

        println!("");
        NodeTable::display(nodes)?;
        Ok(())
    }
}
