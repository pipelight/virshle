use crate::cloud_hypervisor::{Vm, VmState};
use human_bytes::human_bytes;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fmt;
use tabled::{
    settings::{object::Columns, Disable, Style},
    Table, Tabled,
};
use uuid::Uuid;

// Error Handling
use log::{log_enabled, Level};
use miette::{IntoDiagnostic, Result};
use virshle_error::VirshleError;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub struct VmTable {
    pub name: String,
    pub vcpu: u64,
    #[tabled(display_with = "display_vram")]
    pub vram: u64,
    #[tabled(display_with = "display_state")]
    pub state: VmState,
    #[tabled(display_with = "display_ips")]
    pub ips: Vec<String>,
    pub uuid: Uuid,
}
impl VmTable {
    async fn from(vm: &Vm) -> Result<Self, VirshleError> {
        let table = VmTable {
            name: vm.name.to_owned(),
            vcpu: vm.vcpu,
            vram: vm.vram,
            state: vm.get_state().await?,
            ips: vm.get_ips().await?,
            uuid: vm.uuid,
        };
        Ok(table)
    }
}

pub fn display_vram(vram: &u64) -> String {
    let res = human_bytes((vram * u64::pow(1024, 2)) as f64);
    format!("{}", res)
}
pub fn display_ips(ips: &Vec<String>) -> String {
    let res = ips.join("\n");
    format!("{}\n", res)
}

pub fn display_state(state: &VmState) -> String {
    let res = match state {
        VmState::NotCreated => "not_created".white().to_string(),
        VmState::Created => "created".blue().to_string(),
        VmState::Running => "running".green().to_string(),
        VmState::Paused => "paused".yellow().to_string(),
        VmState::Shutdown => "shutdown".red().to_string(),
        VmState::BreakPoint => "breakpoint".white().to_string(),
    };
    format!("{}", res)
}

impl VmTable {
    pub async fn display(items: Vec<VmTable>) -> Result<()> {
        if log_enabled!(Level::Info) {
            let mut res = Table::new(&items);
            res.with(Style::rounded());
            println!("{}", res);
        } else if log_enabled!(Level::Warn) {
            let mut res = Table::new(&items);
            res.with(Style::rounded());
            res.with(Disable::column(Columns::last()));
            println!("{}", res);
        } else {
            let mut res = Table::new(&items);
            res.with(Disable::column(Columns::last()));
            res.with(Disable::column(Columns::last()));
            res.with(Style::rounded());
            println!("{}", res);
        }
        Ok(())
    }
}
impl Vm {
    pub async fn display(items: Vec<Vm>) -> Result<()> {
        let mut table: Vec<VmTable> = vec![];
        for e in items {
            table.push(VmTable::from(&e).await?);
        }
        VmTable::display(table).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_display_state() -> Result<()> {
        println!("\n{}", display_state(&VmState::Running));
        Ok(())
    }
    #[tokio::test]
    async fn display_mock() -> Result<()> {
        // Get vms
        let vms = vec![
            VmTable {
                name: "TestOs".to_owned(),
                vcpu: 2,
                vram: 4_200_000,
                state: VmState::Created,
                uuid: Uuid::new_v4(),
                ips: vec![],
            },
            VmTable {
                name: "NixOs".to_owned(),
                vcpu: 2,
                vram: 4_200_000,
                state: VmState::Running,
                uuid: Uuid::new_v4(),
                ips: vec![],
            },
        ];

        println!("");
        VmTable::display(vms).await?;
        Ok(())
    }
}
