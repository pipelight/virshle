use super::Vm;

use serde::{Deserialize, Serialize};
use tabled::{Table, Tabled};

// Cloud Hypervisor
use hyper::{Request, StatusCode};
use vmm::api::VmInfoResponse;
use vmm::{
    vm::VmState as ChVmState,
    vm_config::{
        // defaults
        default_console,
        default_serial,

        CpusConfig,
        DiskConfig as ChDiskConfig,
        MemoryConfig,
        NetConfig,
        RngConfig,
        VmConfig,
    },
};

use std::fs;

// Http
use crate::http_cli::Connection;

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::{prelude::*, *};
use sea_orm::{prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult};

// Global configuration
use crate::config::MANAGED_DIR;

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};
use virshle_error::{LibError, VirshleError};

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
            let vm: Vm = serde_json::from_value(e.definition)?;
            vms.push(vm)
        }
        Ok(vms)
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
            let vm: Vm = serde_json::from_value(record.definition)?;
            return Ok(vm);
        } else {
            let message = format!("Could not find a vm with the name: {}", name);
            return Err(LibError::new(&message, "").into());
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
            let vm: Vm = serde_json::from_value(record.definition)?;
            return Ok(vm);
        } else {
            let message = format!("Could not find a vm with the uuid: {}", uuid);
            return Err(LibError::new(&message, "").into());
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub enum VmState {
    NotCreated,
    Created,
    Running,
    Shutdown,
    Paused,
    BreakPoint,
}
impl From<ChVmState> for VmState {
    fn from(ch_vm_state: ChVmState) -> Self {
        let res = match ch_vm_state {
            ChVmState::Created => VmState::Created,
            ChVmState::Running => VmState::Running,
            ChVmState::Shutdown => VmState::Shutdown,
            ChVmState::Paused => VmState::Paused,
            ChVmState::BreakPoint => VmState::BreakPoint,
        };
        return res;
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
     * Should be renamed to get_info();
     *
     */
    pub async fn get_info(&self) -> Result<(), VirshleError> {
        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string() + ".sock";
        let endpoint = "/api/v1/vm.info";

        let conn = Connection::open(&socket).await?;

        let response = conn.get(endpoint).await?;
        let data = &response.to_string().await?;
        println!("{}", data);

        let data: VmInfoResponse = serde_json::from_str(&data)?;

        Ok(())
    }
    /*
     * Should be renamed to get_info();
     *
     */
    pub async fn get_state(&self) -> Result<VmState, VirshleError> {
        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string() + ".sock";
        let endpoint = "/api/v1/vm.info";

        let conn = Connection::open(&socket).await;

        let state = match conn {
            Ok(v) => {
                let response = v.get(endpoint).await?;
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
        Vm::get_by_name("default_xs").await?.update().await?;
        Ok(())
    }

    // #[tokio::test]
    async fn fetch_vms() -> Result<()> {
        let items = Vm::get_all().await?;
        println!("{:#?}", items);
        Ok(())
    }
}
