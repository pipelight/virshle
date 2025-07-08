use super::{Vm, VmNet};
use crate::display::VmTable;

use uuid::Uuid;

use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

// Cloud Hypervisor
use crate::cli::VmArgs;
use crate::cloud_hypervisor::vmm_types::{VmConfig, VmInfoResponse, VmState};

// Ips
use crate::config::VirshleConfig;
use crate::network::dhcp::{DhcpType, KeaDhcp};
use std::net::IpAddr;

use std::str::FromStr;

use hyper::{Request, StatusCode};

use std::fs;

// Rest Api
use crate::api::rest::{GetManyVmArgs, GetVmArgs};

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
            if vm.get_state().await? == *state {
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
            let vm: Vm = serde_json::from_value(record.definition)?;
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
    pub ips: Vec<IpAddr>,
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
    /*
     * Return vm's disks directory path.
     */
    pub fn get_disk_dir(&self) -> Result<String, VirshleError> {
        let path = format!("{MANAGED_DIR}/vm/{}/disk", self.uuid);
        Ok(path)
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
    /*
     * Return vm socket path.
     */
    pub fn get_socket(&self) -> Result<String, VirshleError> {
        let path = format!("{MANAGED_DIR}/vm/{}/ch.sock", self.uuid);
        Ok(path)
    }
    pub fn get_socket_uri(&self) -> Result<String, VirshleError> {
        let path = format!("unix://{MANAGED_DIR}/vm/{}/ch.sock", self.uuid);
        Ok(path)
    }

    /// Return structured informations from the vm hypervisor.
    pub async fn get_ch_info(&self) -> Result<VmInfoResponse, VirshleError> {
        let endpoint = "/api/v1/vm.info";

        let mut conn = Connection::from(self);
        conn.open().await?;

        let mut rest = RestClient::from(&mut conn);
        let response = rest.get(endpoint).await?;

        let data = &response.to_string().await?;
        println!("{}", data);

        let data: VmInfoResponse = serde_json::from_str(&data)?;
        Ok(data)
    }
    /// Return structured informations from the vm hypervisor.
    pub async fn ping_ch(&self) -> Result<(), VirshleError> {
        let endpoint = "/api/v1/vm.info";

        let mut conn = Connection::from(self);
        conn.open().await?;

        let mut rest = RestClient::from(&mut conn);
        rest.ping_url("/api/v1/vmm.ping");
        rest.ping().await
    }

    /// Return vm state and ips.
    pub async fn get_info(&self) -> Result<VmInfo, VirshleError> {
        let res = VmInfo {
            state: self.get_state().await?,
            ips: self.get_ips().await?,
            account_uuid: self.get_account_uuid().await.ok(),
        };
        Ok(res)
    }
    /// Return vm state,
    /// or the default state if couldn't connect to vm.
    pub async fn get_state(&self) -> Result<VmState, VirshleError> {
        let endpoint = "/api/v1/vm.info";

        let mut conn = Connection::from(self);
        let mut rest = RestClient::from(&mut conn);

        let state = match rest.open().await {
            Ok(v) => {
                let response = rest.get(endpoint).await?;
                let status = response.status();
                match status {
                    StatusCode::OK => {
                        let data = &response.to_string().await?;
                        let data: VmInfoResponse = serde_json::from_str(&data)?;
                        VmState::from(data.state)
                    }
                    StatusCode::INTERNAL_SERVER_ERROR => VmState::NotCreated,
                    _ => VmState::NotCreated,
                }
            }
            Err(_) => VmState::NotCreated,
        };
        Ok(state)
    }
    /// Return vm ips,
    /// or an empty vec if nothing found.
    pub async fn get_ips(&self) -> Result<Vec<IpAddr>, VirshleError> {
        let mut ips: Vec<IpAddr> = vec![];
        match VirshleConfig::get()?.dhcp {
            Some(DhcpType::Kea(kea_dhcp)) => {
                let hostname = format!("vm-{}", &self.name);
                let leases = KeaDhcp::get_leases_by_hostname(&hostname).await?;
                ips = leases.iter().map(|e| e.address).collect();
            }
            _ => {}
        };
        Ok(ips)
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
