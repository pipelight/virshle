use crate::config::{Node, NodeInfo, NodeState};
use crate::connection::Uri;

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
    pub state: NodeState,
    pub vm: i64,
    #[tabled(display = "display_some_num")]
    pub cpu: Option<u64>,
    #[tabled(display = "display_some_vram")]
    pub ram: Option<u64>,
}

impl NodeTable {
    async fn from(e: &NodeInfo) -> Result<Self, VirshleError> {
        let table = NodeTable {
            name: e.host_info.name.to_owned(),
            cpu: Some(e.host_info.cpu.number),
            ram: Some(e.host_info.ram.total),
            vm: e.virshle_info.num_vm,
            state: NodeState::Running,
        };
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
    pub async fn display(items: HashMap<Node, Option<NodeInfo>>) -> Result<(), VirshleError> {
        let mut table: Vec<NodeTable> = vec![];
        for (node, node_info) in items {
            let mut e;
            if let Some(node_info) = node_info {
                e = NodeTable::from(&node_info).await?;
            } else {
                e = NodeTable::from(&NodeInfo::default()).await?;
                e.state = NodeState::Unreachable;
            }
            table.push(e);
        }
        NodeTable::display(table)?;
        Ok(())
    }
}

pub fn display_state(state: &NodeState) -> String {
    let icon = "●";
    let res = match state {
        NodeState::Running => format!("{} Running", icon).green().to_string(),
        NodeState::Unreachable => format!("{} Unreachable", icon).white().to_string(),
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
                cpu: Some(4),
                ram: Some(4 * u64::pow(1024, 4)),
                state: NodeState::Running,
                vm: 2,
            },
            NodeTable {
                name: "node_2".to_owned(),
                cpu: Some(16),
                ram: Some(30 * u64::pow(1024, 4)),
                state: NodeState::Unreachable,
                vm: 0,
            },
        ];

        println!("");
        NodeTable::display(nodes)?;
        Ok(())
    }
}
