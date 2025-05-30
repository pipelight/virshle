use super::{Vm, VmNet};

use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

// Cloud Hypervisor
use crate::cloud_hypervisor::vmm_types::{VmConfig, VmInfoResponse, VmState};

use hyper::{Request, StatusCode};

use std::fs;

// Http
use crate::connection::{Connection, ConnectionHandle, UnixConnection};
use crate::http_request::{Rest, RestClient};

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::{prelude::*, *};
use sea_orm::{prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult};

// Global configuration
use crate::config::MANAGED_DIR;

// Error Handling
use log::{debug, info, trace, warn};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

impl Vm {
    /*
     * Get all Vm from virshle database.
     */
    pub async fn get_all() -> Result<Vec<Vm>, VirshleError> {
        let db = connect_db().await?;
        let records: Vec<database::entity::vm::Model> =
            database::prelude::Vm::find().all(&db).await?;

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
                    let err = WrapError::builder()
                        .msg("Couldn't convert database record to valid resources")
                        .help("")
                        .origin(VirshleError::from(e).into())
                        .build();
                    return Err(err.into());
                }
            };
            vms.push(vm)
        }
        Ok(vms)
    }
    pub async fn get_by_state(state: VmState) -> Result<Vec<Vm>, VirshleError> {
        let vms = Self::get_all().await?;
        let mut vm_w_state: Vec<Vm> = vec![];
        for vm in vms {
            if vm.get_state().await? == state {
                vm_w_state.push(vm);
            }
        }
        Ok(vm_w_state)
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
    /*
     * Return vm info
     */
    pub async fn get_info(&self) -> Result<VmInfoResponse, VirshleError> {
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

    pub fn is_attach(&self) -> Result<bool, VirshleError> {
        if let Some(config) = &self.config {
            return Ok(config.attach);
        } else {
            return Ok(false);
        };
    }

    /*
     * Should be renamed to get_info();
     *
     */
    pub fn get_state_sync(&self) -> Result<VmState, VirshleError> {
        futures::executor::block_on(self.get_state())
    }

    pub async fn get_state(&self) -> Result<VmState, VirshleError> {
        let endpoint = "/api/v1/vm.info";

        let mut conn = Connection::from(self);

        let state = match conn.open().await {
            Ok(v) => {
                let mut rest = RestClient::from(&mut conn);
                let response = rest.get(endpoint).await?;
                let status = response.status();
                match status {
                    StatusCode::INTERNAL_SERVER_ERROR => VmState::NotCreated,
                    StatusCode::OK => {
                        let data = &response.to_string().await?;
                        let data: VmInfoResponse = serde_json::from_str(&data)?;
                        VmState::from(data.state)
                    }
                    _ => VmState::NotCreated,
                }
            }
            Err(_) => VmState::NotCreated,
        };
        Ok(state)
    }

    pub async fn get_ips(&self) -> Result<Vec<String>, VirshleError> {
        let ips = vec![];
        Ok(ips)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    // #[tokio::test]
    async fn fetch_info() -> Result<()> {
        Vm::get_by_name("default_xs").await?.get_info().await?;
        Ok(())
    }

    // #[tokio::test]
    async fn fetch_vms() -> Result<()> {
        let items = Vm::get_all().await?;
        println!("{:#?}", items);
        Ok(())
    }
}
