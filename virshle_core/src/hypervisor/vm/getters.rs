use super::{Vm, VmNet, VmTemplate};
use crate::display::VmTable;
use crate::hypervisor::disk;
use crate::hypervisor::network::dhcp::Lease;

use uuid::Uuid;

use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

// Cloud Hypervisor
use crate::cli::VmArgs;
use crate::hypervisor::vmm::types::{VmConfig, VmInfoResponse, VmState};

// Ips
use crate::config::Config;
use crate::hypervisor::network::dhcp::{DhcpType, KeaDhcp};

// Network primitives
use crate::hypervisor::network::utils;
use macaddr::MacAddr6;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use std::str::FromStr;

use hyper::{Request, StatusCode};

use std::fs;

// Rest Api
use crate::rest_api::{GetManyVmArgs, GetVmArgs};

// Http
use crate::connection::{Connection, ConnectionHandle, UnixConnection};
use crate::http_request::{Rest, RestClient};

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::prelude;
use sea_orm::{prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult};

// Global configuration
use crate::config::MANAGED_DIR;

// Error Handling
use log::{debug, error, info, trace, warn};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

impl VmTemplate {
    pub fn get_all() -> Result<Vec<VmTemplate>, VirshleError> {
        let config = Config::get()?;
        config.get_templates()
    }
    pub fn get_by_name(name: &str) -> Result<Self, VirshleError> {
        let templates = Self::get_all()?;
        let res: Vec<VmTemplate> = templates.into_iter().filter(|e| e.name == name).collect();
        match res.first() {
            Some(v) => Ok(v.to_owned()),
            None => {
                let message = format!("Couldn't find a vm_template with the name: {}", name);
                let help = "Are you sure this vm template exist?";
                return Err(LibError::builder().msg(&message).help(help).build().into());
            }
        }
    }
}

impl Vm {
    pub fn get(&self) -> VmGetMethods {
        let conn = Connection::from(self);
        let mut rest = RestClient::from(conn);
        rest.ping_url("/vmm.ping");
        VmGetMethods {
            vm: self,
            client: rest,
        }
    }
}
pub struct VmGetMethods<'a> {
    vm: &'a Vm,
    client: RestClient,
}
impl VmGetMethods<'_> {
    /// Return vm state,
    /// or the default state if couldn't connect to vm.
    pub async fn state(&mut self) -> Result<VmState, VirshleError> {
        let endpoint = "/api/v1/vm.info";
        let response = self.client.get(endpoint).await?;
        let state = match response.status() {
            StatusCode::OK => {
                let data = &response.to_string().await?;
                let data: VmInfoResponse = serde_json::from_str(&data)?;
                VmState::from(data.state)
            }
            StatusCode::INTERNAL_SERVER_ERROR => VmState::NotCreated,
            _ => VmState::NotCreated,
        };
        Ok(state)
    }
}

impl Vm {
    pub async fn get_all() -> Result<Vec<Vm>, VirshleError> {
        let db = connect_db().await?;
        let records: Vec<database::entity::vm::Model> = database::prelude::Vm::find()
            .order_by_asc(database::entity::vm::Column::CreatedAt)
            .all(&db)
            .await?;

        let mut vms: Vec<Vm> = vec![];
        for e in records {
            let res: Result<Vm, serde_json::Error> = serde_json::from_value(e.definition);
            let vm: Vm = match res {
                Ok(mut v) => {
                    // Populate struct with database id.
                    v.id = Some(e.id as u64);
                    v.created_at = e.created_at;
                    v.updated_at = e.updated_at;
                    v
                }
                Err(e) => {
                    let message = "Couldn't convert database record to valid resources";
                    let err = WrapError::builder()
                        .msg(message)
                        .help("")
                        .origin(VirshleError::from(e).into())
                        .build();
                    error!("{}", message);
                    return Err(err.into());
                }
            };
            vms.push(vm)
        }
        Ok(vms)
    }
    pub async fn filter_by_state(vms: Vec<Vm>, state: &VmState) -> Result<Vec<Vm>, VirshleError> {
        let mut vm_by_state: Vec<Vm> = vec![];
        for vm in vms {
            if vm.get().state().await? == *state {
                vm_by_state.push(vm);
            }
        }
        Ok(vm_by_state)
    }

    // Return VMs associated with a specific account on node.
    pub async fn get_by_account(account_uuid: &Uuid) -> Result<Vec<Vm>, VirshleError> {
        let db = connect_db().await?;

        let account: Option<database::entity::account::Model> = database::prelude::Account::find()
            .filter(database::entity::account::Column::Uuid.eq(account_uuid.to_string()))
            .one(&db)
            .await?;

        if let Some(account) = account {
            let records: Vec<database::entity::vm::Model> = account
                .find_related(database::entity::prelude::Vm)
                .order_by_asc(database::entity::vm::Column::CreatedAt)
                .all(&db)
                .await?;

            let mut vms: Vec<Vm> = vec![];
            for e in records {
                let res: Result<Vm, serde_json::Error> = serde_json::from_value(e.definition);
                let vm: Vm = match res {
                    Ok(mut v) => {
                        // Populate struct with database id.
                        v.id = Some(e.id as u64);
                        v.created_at = e.created_at;
                        v.updated_at = e.updated_at;
                        v
                    }
                    Err(e) => {
                        let message = "Couldn't convert database record to valid resources";
                        let err = WrapError::builder()
                            .msg(message)
                            .help("")
                            .origin(VirshleError::from(e).into())
                            .build();
                        error!("{}", message);
                        return Err(err.into());
                    }
                };
                vms.push(vm)
            }
            Ok(vms)
        } else {
            // No VM associated with account on this node.
            // TODO: Should maybe return an error.
            Ok(vec![])
        }
    }

    pub async fn get_many_by_args(args: &GetManyVmArgs) -> Result<Vec<Vm>, VirshleError> {
        // Filter by account
        let vms: Vec<Vm> = if let Some(account_uuid) = &args.account_uuid {
            Vm::get_by_account(account_uuid).await?
        } else {
            Vm::get_all().await?
        };
        // Filter by state
        let vms: Vec<Vm> = if let Some(vm_state) = &args.vm_state {
            Vm::filter_by_state(vms, &vm_state).await?
        } else {
            vms
        };
        Ok(vms)
    }
    pub async fn get_by_args(args: &GetVmArgs) -> Result<Vm, VirshleError> {
        if let Some(id) = args.id {
            let vm = Vm::get_by_id(&id).await?;
            Ok(vm)
        } else if let Some(name) = &args.name {
            let vm = Vm::get_by_name(&name).await?;
            Ok(vm)
        } else if let Some(uuid) = &args.uuid {
            let vm = Vm::get_by_uuid(&uuid).await?;
            Ok(vm)
        } else {
            let message = format!("Couldn't find vm.");
            let help = format!("Are you sure the vm exists on this node?");
            Err(LibError::builder().msg(&message).help(&help).build().into())
        }
    }
    /*
     * Get a Vm definition from its name.
     */
    pub async fn get_by_name(name: &str) -> Result<Self, VirshleError> {
        // Retrive from database
        let db = connect_db().await.unwrap();
        let record = database::prelude::Vm::find()
            .filter(database::entity::vm::Column::Name.eq(name))
            .one(&db)
            .await?;

        if let Some(record) = record {
            let mut vm: Vm = serde_json::from_value(record.definition)?;
            // Populate struct with database id.
            vm.id = Some(record.id as u64);
            return Ok(vm);
        } else {
            let message = format!("Couldn't find a vm with the name: {}", name);
            let help = "Are you sure this vm exist?";
            return Err(LibError::builder().msg(&message).help(help).build().into());
        }
    }
    /*
     * Get a Vm definition from its uuid.
     */
    pub async fn get_by_uuid(uuid: &Uuid) -> Result<Self, VirshleError> {
        // Retrive from database
        let db = connect_db().await.unwrap();
        let record = database::prelude::Vm::find()
            .filter(database::entity::vm::Column::Uuid.eq(uuid.to_string()))
            .one(&db)
            .await?;

        if let Some(record) = record {
            let mut vm: Vm = serde_json::from_value(record.definition)?;
            // Populate struct with database id.
            vm.id = Some(record.id as u64);
            return Ok(vm);
        } else {
            let message = format!("Couldn't find a vm with the uuid: {}", uuid);

            let help = "Are you sure this vm exist?";
            return Err(LibError::builder().msg(&message).help(help).build().into());
        }
    }
    /*
     * Get a Vm definition from its id.
     */
    pub async fn get_by_id(id: &u64) -> Result<Self, VirshleError> {
        // Retrive from database
        let db = connect_db().await.unwrap();
        let record = database::prelude::Vm::find()
            .filter(database::entity::vm::Column::Id.eq(id.clone()))
            .one(&db)
            .await?;

        if let Some(record) = record {
            let mut vm: Vm = serde_json::from_value(record.definition)?;
            // Populate struct with database id.
            vm.id = Some(record.id as u64);
            return Ok(vm);
        } else {
            let message = format!("Couldn't find a vm with the id: {}", id);
            let help = "Are you sure this vm exist?";
            return Err(LibError::builder().msg(&message).help(help).build().into());
        }
    }
}

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

    /*
     * Return path where to mount vm pipelight-init disk to.
     *
     * This path is used to provision a pipelight-init disk (cloud-init alternative)
     * with user defined data, mainly:
     * - network interface ips (Ipv4 / Ipv6)
     * - hostname
     */
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
            state: self.get().state().await?,
            leases: self.get().leases().await.ok(),
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
    use std::path::PathBuf;

    // #[tokio::test]
    async fn fetch_info() -> Result<()> {
        Vm::get_by_name("vm-default-xs").await?.get_info().await?;
        Ok(())
    }

    // #[tokio::test]
    async fn fetch_vms() -> Result<()> {
        let items = Vm::get_all().await?;
        println!("{:#?}", items);
        Ok(())
    }
}
