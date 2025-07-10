use crate::config::{HostCpu, HostDisk, HostRam, Node, NodeInfo};
use crate::connection::{ConnectionState, Uri};

use super::utils::*;
use crate::cloud_hypervisor::{Vm, VmState};

use human_bytes::human_bytes;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use tabled::{
    settings::{
        disable::Remove, location::ByColumnName, object::Columns, themes::BorderCorrection, Panel,
        Style,
    },
    Table, Tabled,
};
use uuid::Uuid;

// Error Handling
use log::{log_enabled, Level};
use miette::{IntoDiagnostic, Result};
use virshle_error::VirshleError;

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq, Tabled)]
pub struct CpuTable {
    pub name: String,
    #[tabled(display = "display_some_num")]
    number: Option<u64>,
    #[tabled(display = "display_some_num")]
    reserved: Option<u64>,
    #[tabled(display = "display_some_percentage_used")]
    percentage_reserved: Option<f64>,
    #[tabled(display = "display_some_percentage_used")]
    usage: Option<f64>,
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
        table.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
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
                number: Some(e.number),
                usage: Some(e.usage),
                reserved: Some(e.reserved),
                percentage_reserved: Some(e.reserved as f64 / e.number as f64 * 100.0),
            };
        } else {
            table = CpuTable {
                name: node.name.clone(),
                number: None,
                usage: None,
                reserved: None,
                percentage_reserved: None,
            };
        }
        Ok(table)
    }
    pub fn display(items: Vec<Self>) -> Result<(), VirshleError> {
        let mut res = Table::new(&items);
        if log_enabled!(Level::Info) || log_enabled!(Level::Debug) || log_enabled!(Level::Trace) {
        } else if log_enabled!(Level::Warn) {
            res.with(Remove::column(ByColumnName::new("usage")));
        } else if log_enabled!(Level::Error) {
            res.with(Remove::column(ByColumnName::new("usage")));
            res.with(Remove::column(ByColumnName::new("reserved")));
        }
        res.with(Style::rounded());
        println!("{}", res);
        Ok(())
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq, Tabled)]
pub struct RamTable {
    pub name: String,
    #[tabled(display = "display_some_bytes")]
    total: Option<u64>,
    #[tabled(display = "display_some_bytes")]
    used: Option<u64>,
    #[tabled(display = "display_some_bytes")]
    free: Option<u64>,
    #[tabled(display = "display_some_bytes")]
    reserved: Option<u64>,
    #[tabled(display = "display_some_ram_percentage_reserved")]
    percentage_reserved: Option<f64>,
    #[tabled(display = "display_some_percentage_used")]
    percentage_used: Option<f64>,
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
        table.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
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
                total: Some(e.total),
                used: Some(e.total - e.free),
                free: Some(e.free),
                reserved: Some(e.reserved),
                percentage_reserved: Some((e.reserved as f64 / e.total as f64) * 100.0),
                percentage_used: Some(((e.total as f64 - e.free as f64) / e.total as f64) * 100.0),
            };
        } else {
            table = RamTable {
                name: node.name.clone(),
                total: None,
                used: None,
                free: None,
                reserved: None,
                percentage_reserved: None,
                percentage_used: None,
            };
        }
        Ok(table)
    }
    pub fn display(items: Vec<Self>) -> Result<(), VirshleError> {
        let mut res = Table::new(&items);
        if log_enabled!(Level::Info) || log_enabled!(Level::Debug) || log_enabled!(Level::Trace) {
            res.with(Style::rounded());
        } else if log_enabled!(Level::Warn) {
            res.with(Remove::column(ByColumnName::new("free")));
            res.with(Remove::column(ByColumnName::new("used")));
        } else if log_enabled!(Level::Error) {
            res.with(Remove::column(ByColumnName::new("free")));
            res.with(Remove::column(ByColumnName::new("used")));
            res.with(Remove::column(ByColumnName::new("total")));
            res.with(Remove::column(ByColumnName::new("reserved")));
            res.with(Style::rounded());
        }
        println!("{}", res);
        Ok(())
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq, Tabled)]
pub struct HostDiskTable {
    pub name: String,
    #[tabled(display = "display_some_bytes")]
    size: Option<u64>,
    #[tabled(display = "display_some_bytes")]
    used: Option<u64>,
    #[tabled(display = "display_some_bytes")]
    available: Option<u64>,
    #[tabled(display = "display_some_bytes")]
    reserved: Option<u64>,
    #[tabled(display = "display_some_percentage_used")]
    percentage_used: Option<f64>,
    #[tabled(display = "display_some_disk_percentage_reserved")]
    percentage_reserved: Option<f64>,
}
impl HostDisk {
    pub async fn display(
        items: HashMap<Node, (ConnectionState, Option<NodeInfo>)>,
    ) -> Result<(), VirshleError> {
        let mut table: Vec<HostDiskTable> = vec![];
        for item in items {
            let e = HostDiskTable::from(&item).await?;
            table.push(e);
        }
        table.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
        HostDiskTable::display(table)?;
        Ok(())
    }
}
impl HostDiskTable {
    async fn from(e: &(Node, (ConnectionState, Option<NodeInfo>))) -> Result<Self, VirshleError> {
        let (node, (state, node_info)) = e;
        let table;
        if let Some(node_info) = node_info {
            let e = node_info.host_info.disk.clone();
            table = HostDiskTable {
                name: node.name.to_owned(),
                size: Some(e.size),
                used: Some(e.used),
                available: Some(e.available),
                reserved: Some(e.reserved),

                percentage_used: Some(e.used as f64 / e.size as f64 * 100.0),
                percentage_reserved: Some(e.reserved as f64 / e.size as f64 * 100.0),
            };
        } else {
            table = HostDiskTable {
                name: node.name.to_owned(),
                size: None,
                used: None,
                available: None,
                reserved: None,
                percentage_used: None,
                percentage_reserved: None,
            };
        }
        Ok(table)
    }
    pub fn display(items: Vec<Self>) -> Result<(), VirshleError> {
        let mut res = Table::new(&items);
        if log_enabled!(Level::Warn)
            || log_enabled!(Level::Info)
            || log_enabled!(Level::Debug)
            || log_enabled!(Level::Trace)
        {
        } else if log_enabled!(Level::Error) {
            res.with(Remove::column(ByColumnName::new("used")));
            res.with(Remove::column(ByColumnName::new("reserved")));
            res.with(Remove::column(ByColumnName::new("available")));
        }
        res.with(Style::rounded());
        println!("{}", res);
        Ok(())
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq, Tabled)]
pub struct NodeTable {
    pub name: String,
    #[tabled(display = "display_state")]
    pub state: ConnectionState,
    #[tabled(display = "display_some_num")]
    pub vm: Option<u64>,
    #[tabled(display = "display_some_num")]
    pub cpu: Option<u64>,
    #[tabled(display = "display_some_ram")]
    pub ram: Option<u64>,
    #[tabled(display = "display_some_ram")]
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
        table.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
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
        if log_enabled!(Level::Info) || log_enabled!(Level::Debug) || log_enabled!(Level::Trace) {
            res.with(Style::rounded());
        } else if log_enabled!(Level::Warn) {
            res.with(Remove::column(ByColumnName::new("ram")));
            res.with(Remove::column(ByColumnName::new("cpu")));
            res.with(Remove::column(ByColumnName::new("disk")));
        } else if log_enabled!(Level::Error) {
            res.with(Remove::column(ByColumnName::new("vm")));
            res.with(Remove::column(ByColumnName::new("ram")));
            res.with(Remove::column(ByColumnName::new("cpu")));
            res.with(Remove::column(ByColumnName::new("disk")));
        }
        res.with(Style::rounded());
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
                ram: Some(30 * u64::pow(1024, 3)),
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
