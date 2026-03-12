use super::info::{HostCpu, HostDisk, HostRam, NodeInfo};
use crate::peer::Peer;
use crate::utils::display;

use virshle_network::{connection::ConnectionState, Uri};

use owo_colors::OwoColorize;

use human_bytes::human_bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Globals
use crate::config::{MAX_CPU_RESERVATION, MAX_DISK_RESERVATION, MAX_RAM_RESERVATION};

// Display
use crate::utils::display::*;
use tabled::{
    settings::{disable::Remove, location::ByColumnName, Style, Width},
    Table, Tabled,
};

// Error Handling
use log::{log_enabled, Level};
use miette::Result;
use virshle_error::VirshleError;

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq, Tabled)]
pub struct CpuTable {
    pub alias: String,
    #[tabled(display("display_some_num"))]
    pub number: Option<u64>,
    #[tabled(
        rename = "%reserved",
        display("Self::display_some_cpu_percentage_reserved")
    )]
    pub percentage_reserved: Option<f64>,
    #[tabled(display("Self::display_some_cpu_reserved", &self.percentage_reserved))]
    pub reserved: Option<u64>,
    #[tabled(display("display::display_some_percentage_used"))]
    pub usage: Option<f64>,
}
impl HostCpu {
    pub async fn display_many(
        items: HashMap<Peer, (ConnectionState, Option<NodeInfo>)>,
    ) -> Result<(), VirshleError> {
        let header = "cpu".blue();
        println!("{}", header);

        let mut table: Vec<CpuTable> = vec![];
        for item in items {
            let e = CpuTable::from(&item).await?;
            table.push(e);
        }
        table.sort_by(|a, b| a.alias.partial_cmp(&b.alias).unwrap());
        CpuTable::display(table)?;
        Ok(())
    }
    pub async fn display(
        item: &(Peer, (ConnectionState, Option<NodeInfo>)),
    ) -> Result<(), VirshleError> {
        let section = "cpu".blue();
        let node = item.0.header()?;
        println!("{section} for {node}");

        let mut table: Vec<CpuTable> = vec![];
        let e = CpuTable::from(&item).await?;
        table.push(e);
        CpuTable::display(table)?;
        Ok(())
    }
}

impl CpuTable {
    async fn from(e: &(Peer, (ConnectionState, Option<NodeInfo>))) -> Result<Self, VirshleError> {
        let (node, (state, node_info)) = e;
        let table;
        if let Some(node_info) = node_info {
            let e = node_info.host_info.cpu.clone();
            table = CpuTable {
                alias: node.alias().unwrap_or(node.did()?),
                number: Some(e.number),
                usage: Some(e.usage),
                reserved: Some(e.reserved),
                percentage_reserved: Some(e.reserved as f64 / e.number as f64 * 100.0),
            };
        } else {
            table = CpuTable {
                alias: node.alias()?.clone(),
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
        } else if log_enabled!(Level::Error) {
            res.with(Remove::column(ByColumnName::new("usage")));
            res.with(Remove::column(ByColumnName::new("percentage_reserved")));
        }
        res.modify(ByColumnName::new("usage"), Width::increase(8));
        res.modify(ByColumnName::new("percentage_reserved"), Width::increase(8));
        res.with(Style::rounded());
        println!("{}", res);
        Ok(())
    }
    pub fn display_some_cpu_percentage_reserved(percentage: &Option<f64>) -> String {
        if let Some(percentage) = percentage {
            Self::display_cpu_percentage_reserved(&percentage)
        } else {
            "".to_owned()
        }
    }
    pub fn display_cpu_percentage_reserved(percentage: &f64) -> String {
        let max = MAX_CPU_RESERVATION;
        let progress = make_progress_bar(percentage, Some(max));
        Self::add_color_reserved(&progress, percentage)
    }
    pub fn add_color_reserved(string: &str, percentage: &f64) -> String {
        if percentage < &200_f64 {
            format!("{}", string.green())
        } else if percentage < &250_f64 {
            format!("{}", string.yellow())
        } else {
            format!("{}", string.red())
        }
    }
    pub fn display_some_cpu_reserved(num: &Option<u64>, percentage: &Option<f64>) -> String {
        if num.is_some() && percentage.is_some() {
            format!(
                "{}",
                Self::add_color_reserved(&num.unwrap().to_string(), &percentage.unwrap())
            )
        } else {
            format!("")
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq, Tabled)]
pub struct RamTable {
    pub alias: String,
    #[tabled(display("display::display_some_bytes"))]
    total: Option<u64>,
    #[tabled(display("display::display_some_bytes"))]
    available: Option<u64>,
    #[tabled(
        rename = "%reserved",
        display("Self::display_some_ram_percentage_reserved")
    )]
    percentage_reserved: Option<f64>,
    #[tabled(display("Self::display_some_ram_reserved", &self.percentage_reserved))]
    reserved: Option<u64>,
    #[tabled(rename = "%used", display("Self::display_some_ram_percentage_used"))]
    percentage_used: Option<f64>,
    #[tabled(display("Self::display_some_ram_used", &self.percentage_used))]
    used: Option<u64>,
}
impl HostRam {
    pub async fn display_many(
        items: HashMap<Peer, (ConnectionState, Option<NodeInfo>)>,
    ) -> Result<(), VirshleError> {
        let header = "ram".blue();
        println!("{}", header);

        let mut table: Vec<RamTable> = vec![];
        for item in items {
            let e = RamTable::from(&item).await?;
            table.push(e);
        }
        table.sort_by(|a, b| a.alias.partial_cmp(&b.alias).unwrap());
        RamTable::display(table)?;
        Ok(())
    }
    pub async fn display(
        item: &(Peer, (ConnectionState, Option<NodeInfo>)),
    ) -> Result<(), VirshleError> {
        let section = "ram".blue();
        let node = item.0.header()?;
        println!("{section} for {node}");

        let mut table: Vec<RamTable> = vec![];
        let e = RamTable::from(&item).await?;
        table.push(e);
        RamTable::display(table)?;
        Ok(())
    }
    pub fn display_some_ram_percentage_reserved(percentage: &Option<f64>) -> String {
        if let Some(percentage) = percentage {
            Self::display_ram_percentage_reserved(&percentage)
        } else {
            "".to_owned()
        }
    }
    pub fn display_ram_percentage_reserved(percentage: &f64) -> String {
        let max = MAX_RAM_RESERVATION;
        let progress = make_progress_bar(percentage, Some(max));
        Self::add_color_reserved(&progress, percentage)
    }
    pub fn display_some_ram_reserved(num: &Option<u64>, percentage: &Option<f64>) -> String {
        if num.is_some() && percentage.is_some() {
            let num = human_bytes(num.unwrap() as f64);
            format!("{}", Self::add_color_reserved(&num, &percentage.unwrap()))
        } else {
            format!("")
        }
    }
    pub fn display_some_ram_percentage_used(percentage: &Option<f64>) -> String {
        if let Some(percentage) = percentage {
            Self::display_ram_percentage_used(&percentage)
        } else {
            "".to_owned()
        }
    }
    pub fn display_ram_percentage_used(percentage: &f64) -> String {
        let max = MAX_RAM_RESERVATION;
        let progress = make_progress_bar(percentage, None);
        Self::add_color_used(&progress, percentage)
    }
    pub fn display_some_ram_used(num: &Option<u64>, percentage: &Option<f64>) -> String {
        if num.is_some() && percentage.is_some() {
            let num = human_bytes(num.unwrap() as f64);
            format!("{}", Self::add_color_used(&num, &percentage.unwrap()))
        } else {
            format!("")
        }
    }
    pub fn add_color_reserved(string: &str, percentage: &f64) -> String {
        if percentage < &100_f64 {
            format!("{}", string.green())
        } else if percentage < &200_f64 {
            format!("{}", string.yellow())
        } else {
            format!("{}", string.red())
        }
    }
    pub fn add_color_used(string: &str, percentage: &f64) -> String {
        if percentage < &50_f64 {
            format!("{}", string.green())
        } else if percentage < &80_f64 {
            format!("{}", string.yellow())
        } else {
            format!("{}", string.red())
        }
    }
}
impl RamTable {
    async fn from(e: &(Peer, (ConnectionState, Option<NodeInfo>))) -> Result<Self, VirshleError> {
        let (node, (state, node_info)) = e;
        let table;
        if let Some(node_info) = node_info {
            let e = node_info.host_info.ram.clone();
            table = RamTable {
                alias: node.alias()?,
                total: Some(e.total),
                used: Some(e.used),
                available: Some(e.available),
                reserved: Some(e.reserved),
                percentage_reserved: Some((e.reserved as f64 / e.total as f64) * 100.0),
                percentage_used: Some((e.used as f64 / e.total as f64) * 100.0),
            };
        } else {
            table = RamTable {
                alias: node.alias()?,
                total: None,
                used: None,
                available: None,
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
        } else if log_enabled!(Level::Warn) {
            res.with(Remove::column(ByColumnName::new("available")));
        } else if log_enabled!(Level::Error) {
            res.with(Remove::column(ByColumnName::new("available")));
            res.with(Remove::column(ByColumnName::new("percentage_used")));
            res.with(Remove::column(ByColumnName::new("percentage_reserved")));
        }
        res.modify(ByColumnName::new("percentage_used"), Width::increase(8));
        res.modify(ByColumnName::new("percentage_reserved"), Width::increase(8));
        res.with(Style::rounded());
        println!("{}", res);
        Ok(())
    }
    pub fn display_some_ram_percentage_reserved(percentage: &Option<f64>) -> String {
        if let Some(percentage) = percentage {
            Self::display_ram_percentage_reserved(&percentage)
        } else {
            "".to_owned()
        }
    }
    pub fn display_ram_percentage_reserved(percentage: &f64) -> String {
        let max = MAX_RAM_RESERVATION;
        let progress = make_progress_bar(percentage, Some(max));
        Self::add_color_reserved(&progress, percentage)
    }

    pub fn display_some_ram_reserved(num: &Option<u64>, percentage: &Option<f64>) -> String {
        if num.is_some() && percentage.is_some() {
            let num = human_bytes(num.unwrap() as f64);
            format!("{}", Self::add_color_reserved(&num, &percentage.unwrap()))
        } else {
            format!("")
        }
    }
    pub fn display_some_ram_percentage_used(percentage: &Option<f64>) -> String {
        if let Some(percentage) = percentage {
            Self::display_ram_percentage_used(&percentage)
        } else {
            "".to_owned()
        }
    }
    pub fn display_ram_percentage_used(percentage: &f64) -> String {
        let max = MAX_RAM_RESERVATION;
        let progress = make_progress_bar(percentage, Some(max));
        Self::add_color_used(&progress, percentage)
    }

    pub fn display_some_ram_used(num: &Option<u64>, percentage: &Option<f64>) -> String {
        if num.is_some() && percentage.is_some() {
            let num = human_bytes(num.unwrap() as f64);
            format!("{}", Self::add_color_used(&num, &percentage.unwrap()))
        } else {
            format!("")
        }
    }
    pub fn add_color_reserved(string: &str, percentage: &f64) -> String {
        if percentage < &80_f64 {
            format!("{}", string.green())
        } else if percentage < &95_f64 {
            format!("{}", string.yellow())
        } else {
            format!("{}", string.red())
        }
    }
    pub fn add_color_used(string: &str, percentage: &f64) -> String {
        if percentage < &80_f64 {
            format!("{}", string.green())
        } else if percentage < &95_f64 {
            format!("{}", string.yellow())
        } else {
            format!("{}", string.red())
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq, Tabled)]
pub struct HostDiskTable {
    pub alias: String,
    #[tabled(display("display::display_some_bytes"))]
    size: Option<u64>,
    #[tabled(display("display::display_some_bytes"))]
    available: Option<u64>,
    #[tabled(
        rename = "%reserved",
        display("Self::display_some_disk_percentage_reserved")
    )]
    percentage_reserved: Option<f64>,
    #[tabled(display("Self::display_some_disk_reserved", &self.percentage_reserved))]
    reserved: Option<u64>,
    #[tabled(rename = "%used", display("Self::display_some_disk_percentage_used"))]
    percentage_used: Option<f64>,
    #[tabled(display("Self::display_some_disk_used", &self.percentage_used))]
    used: Option<u64>,
}
impl HostDisk {
    pub async fn display_many(
        items: HashMap<Peer, (ConnectionState, Option<NodeInfo>)>,
    ) -> Result<(), VirshleError> {
        let header = "disk".blue();
        println!("{}", header);

        let mut table: Vec<HostDiskTable> = vec![];
        for item in items {
            let e = HostDiskTable::from(&item)?;
            table.push(e);
        }
        table.sort_by(|a, b| a.alias.partial_cmp(&b.alias).unwrap());
        HostDiskTable::display(table)?;
        Ok(())
    }
    pub async fn display(
        item: &(Peer, (ConnectionState, Option<NodeInfo>)),
    ) -> Result<(), VirshleError> {
        let section = "disk".blue();
        let node = item.0.header()?;
        println!("{section} for {node}");

        let mut table: Vec<HostDiskTable> = vec![];
        let e = HostDiskTable::from(item)?;
        table.push(e);
        HostDiskTable::display(table)?;
        Ok(())
    }
}
impl HostDiskTable {
    fn from(
        e: &(Peer, (ConnectionState, Option<NodeInfo>)),
    ) -> Result<HostDiskTable, VirshleError> {
        let (node, (state, node_info)) = e;
        let table;
        if let Some(node_info) = node_info {
            let e = node_info.host_info.disk.clone();
            table = HostDiskTable {
                alias: node.alias()?.to_owned(),
                size: Some(e.size),
                used: Some(e.used),
                available: Some(e.available),
                reserved: Some(e.reserved),

                percentage_used: Some(e.used as f64 / e.size as f64 * 100.0),
                percentage_reserved: Some(e.reserved as f64 / e.size as f64 * 100.0),
            };
        } else {
            table = HostDiskTable {
                alias: node.alias()?.to_owned(),
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
}
impl HostDiskTable {
    pub fn display(items: Vec<Self>) -> Result<(), VirshleError> {
        let mut res = Table::new(&items);

        if log_enabled!(Level::Info) || log_enabled!(Level::Debug) || log_enabled!(Level::Trace) {
        } else if log_enabled!(Level::Warn) {
            res.with(Remove::column(ByColumnName::new("available")));
        } else if log_enabled!(Level::Error) {
            res.with(Remove::column(ByColumnName::new("available")));
            res.with(Remove::column(ByColumnName::new("percentage_used")));
            res.with(Remove::column(ByColumnName::new("percentage_reserved")));
        }
        res.modify(ByColumnName::new("percentage_used"), Width::increase(8));
        res.modify(ByColumnName::new("percentage_reserved"), Width::increase(8));
        res.with(Style::rounded());
        println!("{}", res);
        Ok(())
    }
    pub fn display_some_disk_percentage_reserved(percentage: &Option<f64>) -> String {
        if let Some(percentage) = percentage {
            Self::display_disk_percentage_reserved(&percentage)
        } else {
            "".to_owned()
        }
    }
    pub fn display_disk_percentage_reserved(percentage: &f64) -> String {
        let max = MAX_DISK_RESERVATION;
        let progress = make_progress_bar(percentage, Some(max));
        Self::add_color_reserved(&progress, percentage)
    }

    pub fn display_some_disk_reserved(num: &Option<u64>, percentage: &Option<f64>) -> String {
        if num.is_some() && percentage.is_some() {
            let num = human_bytes(num.unwrap() as f64);
            format!("{}", Self::add_color_reserved(&num, &percentage.unwrap()))
        } else {
            format!("")
        }
    }
    pub fn display_some_disk_percentage_used(percentage: &Option<f64>) -> String {
        if let Some(percentage) = percentage {
            Self::display_disk_percentage_used(&percentage)
        } else {
            "".to_owned()
        }
    }
    pub fn display_disk_percentage_used(percentage: &f64) -> String {
        let max = MAX_DISK_RESERVATION;
        let progress = make_progress_bar(percentage, Some(max));
        Self::add_color_used(&progress, percentage)
    }

    pub fn display_some_disk_used(num: &Option<u64>, percentage: &Option<f64>) -> String {
        if num.is_some() && percentage.is_some() {
            let num = human_bytes(num.unwrap() as f64);
            format!("{}", Self::add_color_used(&num, &percentage.unwrap()))
        } else {
            format!("")
        }
    }
    pub fn add_color_reserved(string: &str, percentage: &f64) -> String {
        if percentage < &80_f64 {
            format!("{}", string.green())
        } else if percentage < &95_f64 {
            format!("{}", string.yellow())
        } else {
            format!("{}", string.red())
        }
    }
    pub fn add_color_used(string: &str, percentage: &f64) -> String {
        if percentage < &80_f64 {
            format!("{}", string.green())
        } else if percentage < &95_f64 {
            format!("{}", string.yellow())
        } else {
            format!("{}", string.red())
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq, Tabled)]
pub struct NodeTable {
    pub alias: String,
    #[tabled(display = "ConnectionState::display")]
    pub state: ConnectionState,
    #[tabled(display = "display::display_some_num")]
    pub vm: Option<u64>,
    #[tabled(display = "display::display_some_num")]
    pub cpu: Option<u64>,
    #[tabled(display = "display::display_some_bytes")]
    pub ram: Option<u64>,
    #[tabled(display = "display::display_some_bytes")]
    pub disk: Option<u64>,
}
impl Peer {
    pub async fn display_many(
        items: HashMap<Peer, (ConnectionState, Option<NodeInfo>)>,
    ) -> Result<(), VirshleError> {
        let mut table: Vec<NodeTable> = vec![];
        for item in items {
            let e = NodeTable::from(&item)?;
            table.push(e);
        }
        table.sort_by(|a, b| a.alias.partial_cmp(&b.alias).unwrap());
        NodeTable::display(table)?;
        Ok(())
    }
    pub async fn display(
        item: &(Peer, (ConnectionState, Option<NodeInfo>)),
    ) -> Result<(), VirshleError> {
        let mut table: Vec<NodeTable> = vec![];
        let e = NodeTable::from(&item)?;
        table.push(e);
        NodeTable::display(table)?;
        Ok(())
    }
    pub fn header(&self) -> Result<String, VirshleError> {
        let alias = self.alias()?.bright_purple().bold().to_string();
        let header: String = match Uri::new(&self.url)? {
            Uri::SshUri(e) => format!(
                "{alias} on {}@{}",
                e.user.yellow().bold(),
                e.host.green().bold()
            ),
            Uri::LocalUri(e) => format!("{alias} on {}", "localhost".green().bold()),
            Uri::TcpUri(e) => format!(
                "{alias} on {}{}",
                e.host.green().bold(),
                e.port.blue().bold()
            ),
        };
        Ok(header)
    }
}

impl NodeTable {
    fn from(e: &(Peer, (ConnectionState, Option<NodeInfo>))) -> Result<Self, VirshleError> {
        let (node, (state, node_info)) = e;
        let table;
        if let Some(node_info) = node_info {
            table = NodeTable {
                alias: node.alias()?.to_owned(),
                cpu: Some(node_info.host_info.cpu.number),
                ram: Some(node_info.host_info.ram.total),
                disk: Some(node_info.host_info.disk.size),
                vm: Some(node_info.virshle_info.num_vm),
                state: state.to_owned(),
            };
        } else {
            table = NodeTable {
                alias: node.alias()?.to_owned(),
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
    pub fn display_w_header(items: Vec<Self>, header: &str) -> Result<(), VirshleError> {
        println!("\n{}", header);
        Self::display(items);
        Ok(())
    }
    pub fn display(items: Vec<Self>) -> Result<(), VirshleError> {
        let mut res = Table::new(&items);
        if log_enabled!(Level::Info) || log_enabled!(Level::Debug) || log_enabled!(Level::Trace) {
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

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn display_mock() -> Result<()> {
        // Get nodes
        let nodes = vec![
            NodeTable {
                alias: "node_1".to_owned(),
                cpu: None,
                ram: None,
                disk: None,
                vm: None,
                state: ConnectionState::Down,
            },
            NodeTable {
                alias: "node_2".to_owned(),
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
