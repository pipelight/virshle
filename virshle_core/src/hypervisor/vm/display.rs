use crate::hypervisor::{DiskInfo, Vm, VmState};
use crate::peer::Peer;
use crate::utils::display;

// Time
use chrono::{NaiveDateTime, TimeDelta, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use tabled::{
    settings::{disable::Remove, location::ByColumnName, Style},
    Table, Tabled,
};
use uuid::Uuid;

// Error Handling
use log::{log_enabled, Level};
use miette::Result;
use virshle_error::VirshleError;

/// A struct that is used to send VM info between peers and display VM info.
/// Peers do not send the raw VM struct over the wire
/// because it lacks a lot of dynamicaly retrievable informations.
/// See it as a snapshot of vm X at time T.
#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub struct VmTable {
    #[tabled(display("display::display_id"))]
    pub id: Option<u64>,
    pub name: String,

    pub vcpu: u64,
    pub vram: String,

    #[tabled(display("VmState::display"))]
    pub state: VmState,

    #[tabled(display("DiskInfo::display_some_vec_short"))]
    pub disk: Option<Vec<DiskInfo>>,
    #[tabled(display("display::display_ips"))]
    pub ips: Option<Vec<IpAddr>>,

    #[tabled(display("display::display_datetime"))]
    pub created_at: NaiveDateTime,
    #[tabled(display("display::display_datetime"))]
    pub updated_at: NaiveDateTime,

    pub uuid: Uuid,
    #[tabled(display("display::display_account_uuid"))]
    pub account_uuid: Option<Uuid>,
}

impl VmTable {
    pub fn display_age(&self) -> Result<String, VirshleError> {
        Ok(display::display_duration(&self.age()?))
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

impl VmTable {
    // Display vms by peer with table header.
    pub async fn display_by_peer(items: &IndexMap<Peer, Vec<VmTable>>) -> Result<(), VirshleError> {
        for (peer, table) in items {
            let header = peer.header()?;
            VmTable::display_w_header(table, &header);
        }
        Ok(())
    }
    pub fn display_w_header(items: &Vec<Self>, header: &str) -> Result<(), VirshleError> {
        println!("\n{}", header);
        Self::display(items);
        Ok(())
    }
    pub fn display(items: &Vec<Self>) -> Result<(), VirshleError> {
        let mut res = Table::new(items);

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
        } else {
            // if log_enabled!(Level::Error)
            // or log not enabled
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
    use crate::utils::testing;
    use miette::IntoDiagnostic;

    #[test]
    fn try_display_state() -> Result<()> {
        println!("\n{}", &VmState::Running.display());
        Ok(())
    }
    #[tokio::test]
    async fn display_mock() -> Result<()> {
        testing::logger()
            .verbosity(tracing::Level::INFO)
            .db(false)
            .set()?;
        // Get vms
        let vms = vec![
            VmTable {
                id: Some(0),
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
                ips: Some(vec!["2001:db8a::1".parse().into_diagnostic()?]),
                ..Default::default()
            },
        ];
        println!("");
        VmTable::display(&vms)?;
        Ok(())
    }
}
