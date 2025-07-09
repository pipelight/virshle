use crate::config::{HostCpu, HostDisk, HostRam, Node, NodeInfo};
use crate::connection::{ConnectionState, Uri};

use super::utils::{
    display_id, display_ips, display_percentage, display_some_num, display_some_vram, display_vram,
};
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

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Tabled)]
pub struct CpuTable {
    pub name: String,
    number: u64,
    usage: u64,
    reserved: u64,
    #[tabled(display = "display_percentage")]
    percentage_reserved: f64,
}
impl HostCpu {
    pub async fn display(
        items: HashMap<Node, (ConnectionState, Option<NodeInfo>)>,
    ) -> Result<(), VirshleError> {
        let mut table: Vec<CpuTable> = vec![];
        for item in items {
            let e = CpuTable::from(&item).await?;
            table.push(e);
        }
        CpuTable::display(table)?;
        Ok(())
    }
}
impl CpuTable {
    async fn from(e: &(Node, (ConnectionState, Option<NodeInfo>))) -> Result<Self, VirshleError> {
        let (node, (state, node_info)) = e;
        let table;
        if let Some(node_info) = node_info {
            let e = node_info.host_info.cpu.clone();
            table = CpuTable {
                name: node.name.clone(),
                number: e.number,
                usage: e.usage,
                reserved: e.reserved,
                percentage_reserved: (e.reserved as f64 / e.number as f64 * 100.0).round() as f64,
            };
        } else {
            table = CpuTable {
                name: node.name.clone(),
                number: 0,
                usage: 0,
                reserved: 0,
                percentage_reserved: 0 as f64,
            };
        }
        Ok(table)
    }
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

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Tabled)]
pub struct RamTable {
    pub name: String,
    #[tabled(display = "display_vram")]
    total: u64,
    #[tabled(display = "display_vram")]
    used: u64,
    #[tabled(display = "display_vram")]
    free: u64,
    #[tabled(display = "display_vram")]
    reserved: u64,
    #[tabled(display = "display_percentage")]
    percentage_reserved: f64,
    #[tabled(display = "display_percentage")]
    percentage_used: f64,
}
impl HostRam {
    pub async fn display(
        items: HashMap<Node, (ConnectionState, Option<NodeInfo>)>,
    ) -> Result<(), VirshleError> {
        let mut table: Vec<RamTable> = vec![];
        for item in items {
            let e = RamTable::from(&item).await?;
            table.push(e);
        }
        RamTable::display(table)?;
        Ok(())
    }
}
impl RamTable {
    async fn from(e: &(Node, (ConnectionState, Option<NodeInfo>))) -> Result<Self, VirshleError> {
        let (node, (state, node_info)) = e;
        let table;
        if let Some(node_info) = node_info {
            let e = node_info.host_info.ram.clone();
            table = RamTable {
                name: node.name.clone(),
                total: e.total,
                used: e.total - e.free,
                free: e.free,
                reserved: e.reserved,
                percentage_reserved: ((e.reserved as f64 / e.total as f64) * 100.0).round() as f64,
                percentage_used: (((e.total as f64 - e.free as f64) / e.total as f64) * 100.0)
                    .round() as f64,
            };
        } else {
            table = RamTable {
                name: node.name.clone(),
                total: 0,
                used: 0,
                free: 0,
                reserved: 0,
                percentage_reserved: 0 as f64,
                percentage_used: 0 as f64,
            };
        }
        Ok(table)
    }
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

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Tabled)]
pub struct DiskTable {
    pub name: String,
    #[tabled(display = "display_vram")]
    size: u64,
    #[tabled(display = "display_vram")]
    used: u64,
    #[tabled(display = "display_vram")]
    available: u64,
    #[tabled(display = "display_percentage")]
    percentage_used: f64,
}
impl HostDisk {
    pub async fn display(
        items: HashMap<Node, (ConnectionState, Option<NodeInfo>)>,
    ) -> Result<(), VirshleError> {
        let mut table: Vec<DiskTable> = vec![];
        for item in items {
            let e = DiskTable::from(&item).await?;
            table.push(e);
        }
        DiskTable::display(table)?;
        Ok(())
    }
}
impl DiskTable {
    async fn from(e: &(Node, (ConnectionState, Option<NodeInfo>))) -> Result<Self, VirshleError> {
        let (node, (state, node_info)) = e;
        let table;
        if let Some(node_info) = node_info {
            let e = node_info.host_info.disk.clone();
            table = DiskTable {
                name: node.name.to_owned(),
                size: e.size,
                used: e.used,
                available: e.available,
                percentage_used: (e.used as f64 / e.size as f64 * 100.0).round() as f64,
            };
        } else {
            table = DiskTable {
                name: node.name.to_owned(),
                size: 0,
                used: 0,
                available: 0,
                percentage_used: 0 as f64,
            };
        }
        Ok(table)
    }
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
    #[tabled(display = "display_some_vram")]
    pub disk: Option<u64>,
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

impl NodeTable {
    async fn from(e: &(Node, (ConnectionState, Option<NodeInfo>))) -> Result<Self, VirshleError> {
        let (node, (state, node_info)) = e;
        let table;
        if let Some(node_info) = node_info {
            table = NodeTable {
                name: node.name.to_owned(),
                cpu: Some(node_info.host_info.cpu.number),
                ram: Some(node_info.host_info.ram.total),
                disk: Some(node_info.host_info.disk.size),
                vm: Some(node_info.virshle_info.num_vm),
                state: state.to_owned(),
            };
        } else {
            table = NodeTable {
                name: node.name.to_owned(),
                cpu: None,
                ram: None,
                disk: None,
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
                disk: None,
                vm: None,
                state: ConnectionState::Down,
            },
            NodeTable {
                name: "node_2".to_owned(),
                cpu: Some(16),
                ram: Some(30 * u64::pow(1024, 4)),
                disk: None,
                vm: Some(2),
                state: ConnectionState::DaemonUp,
            },
        ];

        println!("");
        NodeTable::display(nodes)?;
        Ok(())
    }
}
