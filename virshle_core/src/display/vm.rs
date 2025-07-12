use super::utils::*;
use crate::cloud_hypervisor::{DiskInfo, Vm, VmState};
use crate::config::Node;
use crate::connection::Uri;

// Time
use chrono::{DateTime, NaiveDateTime, Utc};

use human_bytes::human_bytes;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;
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

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub struct VmTable {
    #[tabled(display("display_id"))]
    pub id: Option<u64>,
    pub name: String,
    pub vcpu: u64,
    #[tabled(display("display_vram"))]
    pub vram: u64,
    #[tabled(display("display_state"))]
    pub state: VmState,
    #[tabled(display("display_ips"))]
    pub ips: Vec<IpAddr>,
    #[tabled(display("display_some_disks_short"))]
    pub disk: Option<Vec<DiskInfo>>,

    #[tabled(display("display_some_datetime"))]
    pub created_at: Option<NaiveDateTime>,
    #[tabled(display("display_some_datetime"))]
    pub updated_at: Option<NaiveDateTime>,

    pub uuid: Uuid,
    #[tabled(display("display_account_uuid"))]
    pub account_uuid: Option<Uuid>,
}

impl VmTable {
    pub async fn from(vm: &Vm) -> Result<Self, VirshleError> {
        let tmp = vm.get_info().await?;
        let table = VmTable {
            id: vm.id,
            name: vm.name.to_owned(),
            vcpu: vm.vcpu,
            vram: vm.vram,
            state: tmp.state,
            ips: tmp.ips,
            disk: Some(DiskInfo::from_vec(&vm.disk)?),
            created_at: Some(vm.created_at),
            updated_at: Some(vm.updated_at),
            uuid: vm.uuid,
            account_uuid: tmp.account_uuid,
        };
        Ok(table)
    }
    pub async fn from_vec(vms: &Vec<Vm>) -> Result<Vec<Self>, VirshleError> {
        let mut table_vms = vec![];
        for vm in vms {
            let e = VmTable::from(&vm).await?;
            table_vms.push(e);
        }
        Ok(table_vms)
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
    pub async fn display_by_nodes(items: HashMap<Node, Vec<Self>>) -> Result<(), VirshleError> {
        // Display vm by nodes with table header
        for (node, table) in items {
            let header = node.get_header()?;
            VmTable::display_w_header(table, &header);
        }

        Ok(())
    }
    pub fn display_w_header(items: Vec<Self>, header: &str) -> Result<(), VirshleError> {
        println!("\n{}", header);
        Self::display(items);
        Ok(())
    }
    pub fn display(items: Vec<Self>) -> Result<(), VirshleError> {
        let mut res = Table::new(&items);
        if log_enabled!(Level::Trace) {
        } else if log_enabled!(Level::Debug) {
        } else if log_enabled!(Level::Info) {
            res.with(Remove::column(ByColumnName::new("account_uuid")));
            res.with(Remove::column(ByColumnName::new("uuid")));
        } else if log_enabled!(Level::Warn) {
            res.with(Remove::column(ByColumnName::new("account_uuid")));
            res.with(Remove::column(ByColumnName::new("uuid")));
            res.with(Remove::column(ByColumnName::new("created_at")));
        } else if log_enabled!(Level::Error) {
            res.with(Remove::column(ByColumnName::new("account_uuid")));
            res.with(Remove::column(ByColumnName::new("uuid")));
            res.with(Remove::column(ByColumnName::new("created_at")));
            res.with(Remove::column(ByColumnName::new("disk")));
            res.with(Remove::column(ByColumnName::new("ips")));
        }
        res.with(Remove::column(ByColumnName::new("updated_at")));
        res.with(Style::rounded());
        println!("{}", res);
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
                ..Default::default()
            },
            VmTable {
                id: None,
                name: "NixOs".to_owned(),
                vcpu: 2,
                vram: 4_200_000,
                state: VmState::Running,
                uuid: Uuid::new_v4(),
                ips: vec![],
                ..Default::default()
            },
        ];

        println!("");
        VmTable::display(vms)?;
        Ok(())
    }
}
