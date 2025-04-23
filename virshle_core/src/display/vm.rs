use super::utils::{display_id, display_ips, display_vram};
use crate::cloud_hypervisor::{Vm, VmState};
use human_bytes::human_bytes;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fmt;
use tabled::{
    settings::{disable::Remove, object::Columns, Style},
    Table, Tabled,
};
use uuid::Uuid;

// Error Handling
use log::{log_enabled, Level};
use miette::{IntoDiagnostic, Result};
use virshle_error::VirshleError;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub struct VmTable {
    #[tabled(display = "display_id")]
    pub id: Option<u64>,
    pub name: String,
    pub vcpu: u64,
    #[tabled(display = "display_vram")]
    pub vram: u64,
    #[tabled(display = "display_state")]
    pub state: VmState,
    #[tabled(display = "display_ips")]
    pub ips: Vec<String>,
    pub uuid: Uuid,
}
impl VmTable {
    async fn from(vm: Vm) -> Result<Self, VirshleError> {
        let table = VmTable {
            id: vm.id,
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
    pub async fn display(items: Vec<Self>) -> Result<()> {
        if log_enabled!(Level::Info) {
            let mut res = Table::new(&items);
            res.with(Style::rounded());
            println!("{}", res);
        } else if log_enabled!(Level::Warn) {
            let mut res = Table::new(&items);
            res.with(Style::rounded());
            res.with(Remove::column(Columns::last()));
            println!("{}", res);
        } else {
            let mut res = Table::new(&items);
            res.with(Remove::column(Columns::last()));
            res.with(Remove::column(Columns::last()));
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
            table.push(VmTable::from(e).await?);
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
                id: None,
                name: "TestOs".to_owned(),
                vcpu: 2,
                vram: 4_200_000,
                state: VmState::Created,
                uuid: Uuid::new_v4(),
                ips: vec![],
            },
            VmTable {
                id: None,
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
