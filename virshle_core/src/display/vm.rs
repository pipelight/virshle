use crate::cloud_hypervisor::Vm;
use human_bytes::human_bytes;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fmt;
use tabled::{
    settings::{object::Columns, Disable, Style},
    Table, Tabled,
};
use uuid::Uuid;

// Cloud Hypervisor
use vmm::vm::VmState;

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
    pub state: Option<VmState>,
    #[tabled(display_with = "display_ips")]
    pub ips: Vec<String>,
    pub uuid: Uuid,
}
impl VmTable {
    async fn from(vm: &Vm) -> Result<Self, VirshleError> {
        let table = VmTable {
            name: vm.name,
            vcpu: vm.vcpu,
            vram: vm.vram,
            state: vm.get_state(),
            ips: vm.get_ips(),
            uuid: vm.uuid,
        };
        Ok(table)
    }
}

pub fn display_vram(vram: &u64) -> String {
    let res = human_bytes((vram * 1024) as f64);
    format!("{}", res)
}
pub fn display_ips(ips: &Vec<String>) -> String {
    let res = ips.join("\n");
    format!("{}\n", res)
}

pub fn display_state(state: &Option<VmState>) -> String {
    let res = match state {
        Some(VmState::Created) => "created".blue().to_string(),
        Some(VmState::Running) => "running".green().to_string(),
        Some(VmState::Paused) => "paused".yellow().to_string(),
        Some(VmState::Shutdown) => "shutdown".red().to_string(),
        Some(VmState::BreakPoint) => "breakpoint".white().to_string(),
        None => "none".to_owned(),
    };
    format!("{}", res)
}

impl Vm {
    pub async fn display(vms: Vec<Vm>) -> Result<()> {
        let mut table: Vec<VmTable> = vec![];
        for vm in vms {
            table.push(VmTable::from(&vm).await?);
        }

        if log_enabled!(Level::Info) {
            let mut res = Table::new(&table);
            res.with(Style::rounded());
            println!("{}", res);
        } else if log_enabled!(Level::Warn) {
            let mut res = Table::new(&table);
            res.with(Style::rounded());
            res.with(Disable::column(Columns::last()));
            println!("{}", res);
        } else {
            let mut res = Table::new(&table);
            res.with(Disable::column(Columns::last()));
            res.with(Disable::column(Columns::last()));
            res.with(Style::rounded());
            println!("{}", res);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_display_state() -> Result<()> {
        println!("\n{}", display_state(&Some(VmState::Running)));
        Ok(())
    }
    #[tokio::test]
    async fn display_mock() -> Result<()> {
        // Get vms
        let vms = vec![
            Vm {
                name: "TestOs".to_owned(),
                vcpu: 2,
                vram: 4_200_000,
                state: Some(VmState::Shutdown),
                uuid: Uuid::new_v4(),
                // ips: vec![],
            },
            Vm {
                name: "NixOs".to_owned(),
                vcpu: 2,
                vram: 4_200_000,
                state: Some(VmState::Running),
                uuid: Uuid::new_v4(),
                // ips: vec![],
            },
        ];

        println!("");
        Vm::display(vms).await?;
        Ok(())
    }
}
