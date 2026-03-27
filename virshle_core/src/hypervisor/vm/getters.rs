use crate::hypervisor::Vm;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Cloud Hypervisor
use crate::hypervisor::{VmState, VmTable};

// Ips
use crate::config::{Config, VmNet, VmTemplate};
use crate::network::dhcp::Lease;
use crate::network::utils;

// Network primitives
use macaddr::MacAddr6;

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::prelude;
use sea_orm::{prelude::*, query::*};

// Global configuration
use crate::config::init::MANAGED_DIR;

// Error Handling
use miette::Result;
use tracing::debug;
use virshle_error::{LibError, VirshleError};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct VmInfo {
    pub state: VmState,
    pub leases: Option<Vec<Lease>>,
    pub account_uuid: Option<Uuid>,
}

/// From cloud-hypervisor
#[derive(Clone, Deserialize, Serialize)]
pub struct VmmPingResponse {
    pub build_version: String,
    pub version: String,
    pub pid: i64,
    pub features: Vec<String>,
}
/*
* Getters.
* Get data from cloud-hypervisor on the file.
* Retrieve in real time everything that would be awkward to keep staticaly in a struct field,
* like vm state (on, off...), dinamicaly assigned ips over a network...
*/
impl Vm {
    /*
     * Return vm network socket path.
     */
    pub fn get_dir(&self) -> Result<String, VirshleError> {
        let path = format!("{MANAGED_DIR}/vm/{}", self.uuid);
        Ok(path)
    }
    /*
     * Return vm network socket path.
     */
    pub fn get_net_socket(&self, net: &VmNet) -> Result<String, VirshleError> {
        let path = format!("{MANAGED_DIR}/vm/{}/net/{}.sock", self.uuid, net.name);
        Ok(path)
    }
    /// Return vm's disks directory path.
    pub fn get_disks_dir(&self) -> Result<String, VirshleError> {
        let path = format!("{MANAGED_DIR}/vm/{}/disk", self.uuid);
        Ok(path)
    }
    /// Get sum of vm disks size.
    pub fn get_disks_size(&self) -> u64 {
        self.disk.iter().map(|e| e.get_size().unwrap_or(0)).sum()
    }

    /// Return path where to mount vm pipelight-init disk to.
    /// This path is used to provision a pipelight-init disk (cloud-init alternative)
    /// with user defined data, mainly:
    /// - network interface ips (Ipv4 / Ipv6)
    /// - hostname
    pub fn get_mount_dir(&self) -> Result<String, VirshleError> {
        let path = format!("{MANAGED_DIR}/vm/{}/tmp", self.uuid);
        Ok(path)
    }
    /// Return vm vsocket path for host guest (ssh) communication.
    pub fn get_vsocket(&self) -> Result<String, VirshleError> {
        let path = format!("{MANAGED_DIR}/vm/{}/ch.vsock", self.uuid);
        Ok(path)
    }

    /// Return vm state and ips.
    pub async fn get_info(&self) -> Result<VmInfo, VirshleError> {
        let res = VmInfo {
            state: self.vmm().api()?.state().await?,
            leases: self.networks().leases().get_all().await.ok(),
            account_uuid: self.get_account_uuid().await.ok(),
        };
        Ok(res)
    }

    pub async fn get_account_uuid(&self) -> Result<Uuid, VirshleError> {
        let db = connect_db().await?;
        let vm_record: Option<database::entity::vm::Model> = database::prelude::Vm::find()
            .filter(database::entity::vm::Column::Uuid.eq(self.uuid.to_string()))
            .order_by_asc(database::entity::vm::Column::CreatedAt)
            .one(&db)
            .await?;

        if let Some(vm_record) = vm_record {
            let account = vm_record.find_related(prelude::Account).one(&db).await?;
            if let Some(account) = account {
                return Ok(Uuid::parse_str(&account.uuid)?);
            }
        }
        let err = LibError::builder()
            .msg("Couldn't find any associated account for this vm.")
            .help("")
            .build();
        Err(err.into())
    }

    pub fn get_default_mac(&self) -> Result<MacAddr6, VirshleError> {
        let mac_address = utils::uuid_to_mac(&self.uuid);
        Ok(mac_address)
    }
}

impl VmTable {
    /// Get sum of vm disks size.
    pub fn get_disks_size(&self) -> u64 {
        let mut size = 0;
        if let Some(disks) = &self.disk {
            size = disks.iter().map(|e| e.size.unwrap_or(0)).sum();
        }
        size
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn fetch_many() -> Result<()> {
        let items = Vm::database().await?.many().get().await?;
        // debug!("{:#?}", items);
        Ok(())
    }
    #[tokio::test]
    async fn fetch_one_info() -> Result<()> {
        let items = Vm::database().await?.many().get().await?;
        if let Some(vm) = items.first() {
            let vm = Vm::database()
                .await?
                .one()
                .uuid(vm.uuid)
                .get()
                .await?
                .get_info()
                .await?;
            println!("{:#?}", vm)
        }
        Ok(())
    }

    #[tokio::test]
    async fn fetch_ips() -> Result<()> {
        let items = Vm::database().await?.many().get().await?;
        if let Some(vm) = items.first() {
            let vm = Vm::database()
                .await?
                .one()
                .uuid(vm.uuid)
                .get()
                .await?
                .networks()
                .leases()
                .get_all()
                .await?;
            println!("{:#?}", vm)
        }
        Ok(())
    }

    #[tokio::test]
    async fn find_by_id() -> Result<()> {
        let items = Vm::database().await?.many().get().await?;
        if let Some(vm) = items.first() {
            let re_vm = Vm::database().await?.one().maybe_id(vm.id).get().await?;
            assert_eq!(vm, &re_vm);
        }
        Ok(())
    }
    #[tokio::test]
    async fn find_by_uuid() -> Result<()> {
        let items = Vm::database().await?.many().get().await?;
        if let Some(vm) = items.first() {
            let re_vm = Vm::database().await?.one().uuid(vm.uuid).get().await?;
            assert_eq!(vm, &re_vm);
        }
        Ok(())
    }
    #[tokio::test]
    async fn find_by_name() -> Result<()> {
        let items = Vm::database().await?.many().get().await?;
        if let Some(vm) = items.first() {
            let re_vm = Vm::database().await?.one().name(&vm.name).get().await?;
            assert_eq!(vm, &re_vm);
        }
        Ok(())
    }
}
