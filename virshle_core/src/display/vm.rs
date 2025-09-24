use super::utils::*;
use crate::cloud_hypervisor::{DiskInfo, Vm, VmState};
use crate::config::Node;
use crate::connection::Uri;

// Time
use crate::cloud_hypervisor::disk::utils;
use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
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
    pub vram: String,

    #[tabled(display("display_state"))]
    pub state: VmState,

    #[tabled(display("display_some_disks_short"))]
    pub disk: Option<Vec<DiskInfo>>,
    #[tabled(display("display_ips"))]
    pub ips: Option<Vec<IpAddr>>,

    #[tabled(display("display_datetime"))]
    pub created_at: NaiveDateTime,
    #[tabled(display("display_datetime"))]
    pub updated_at: NaiveDateTime,

    pub uuid: Uuid,
    #[tabled(display("display_account_uuid"))]
    pub account_uuid: Option<Uuid>,
}

impl VmTable {
    pub fn display_age(&self) -> Result<String, VirshleError> {
        Ok(display_duration(&self.age()?))
    }
    pub fn age(&self) -> Result<TimeDelta, VirshleError> {
        let now: NaiveDateTime = Utc::now().naive_utc();
        let created = self.created_at;
        let age = now - created;
        Ok(age)
    }
}
impl VmTable {
    pub async fn from(vm: &Vm) -> Result<Self, VirshleError> {
        let tmp = vm.get_info().await?;

        let ips = tmp
            .leases
            .map(|inner| inner.iter().map(|e| e.address).collect());

        let table = VmTable {
            id: vm.id,
            name: vm.name.to_owned(),
            vcpu: vm.vcpu,
            vram: vm.vram.clone(),
            state: tmp.state,
            ips,
            disk: Some(DiskInfo::from_vec(&vm.disk)?),
            created_at: vm.created_at,
            updated_at: vm.updated_at,
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

        // Ips
        let some_ips: Vec<_> = items
            .clone()
            .into_iter()
            .map(|e| e.ips)
            // Has some leases
            .filter(|e| e.is_some())
            .collect();
        if some_ips.is_empty() {
            res.with(Remove::column(ByColumnName::new("ips")));
        }

        // Account
        let some_account: Vec<_> = items
            .into_iter()
            .map(|e| e.account_uuid)
            // Belongs to an account
            .filter(|e| e.is_some())
            .collect();
        if some_account.is_empty() {
            res.with(Remove::column(ByColumnName::new("account_uuid")));
        }

        if log_enabled!(Level::Trace) {
        } else if log_enabled!(Level::Debug) {
        } else if log_enabled!(Level::Info) {
            res.with(Remove::column(ByColumnName::new("account_uuid")));
            res.with(Remove::column(ByColumnName::new("uuid")));
        } else if log_enabled!(Level::Warn) {
            res.with(Remove::column(ByColumnName::new("account_uuid")));
            res.with(Remove::column(ByColumnName::new("uuid")));
            res.with(Remove::column(ByColumnName::new("created_at")));
            res.with(Remove::column(ByColumnName::new("updated_at")));
        } else if log_enabled!(Level::Error) {
            res.with(Remove::column(ByColumnName::new("account_uuid")));
            res.with(Remove::column(ByColumnName::new("uuid")));
            res.with(Remove::column(ByColumnName::new("created_at")));
            res.with(Remove::column(ByColumnName::new("updated_at")));
            res.with(Remove::column(ByColumnName::new("disk")));
            res.with(Remove::column(ByColumnName::new("ips")));
        }
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
                vram: "4GiB".to_owned(),
                state: VmState::Created,
                uuid: Uuid::new_v4(),
                ips: None,
                ..Default::default()
            },
            VmTable {
                id: None,
                name: "NixOs".to_owned(),
                vcpu: 2,
                vram: "4GiB".to_owned(),
                state: VmState::Running,
                uuid: Uuid::new_v4(),
                ips: None,
                ..Default::default()
            },
        ];

        println!("");
        VmTable::display(vms)?;
        Ok(())
    }
}
