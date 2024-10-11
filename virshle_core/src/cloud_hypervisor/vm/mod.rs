pub mod from;
pub mod getters;
pub mod to;

// Cloud Hypervisor
use uuid::Uuid;
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

use hyper::Request;
use serde::{Deserialize, Serialize};

use pipelight_exec::Process;
use std::io::Write;
use tabled::{
    settings::{object::Columns, Disable, Style},
    Table, Tabled,
};

use super::template;

// Http
use crate::http_cli::Connection;

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::{prelude::*, *};
use sea_orm::{prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult};

use crate::config::MANAGED_DIR;
use crate::display::vm::*;

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};
use virshle_error::{LibError, VirshleError};

use serde_json::{from_slice, Value};
use std::fs;

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

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct VirshleVmConfig {
    autostart: bool,
}
impl Default for VirshleVmConfig {
    fn default() -> Self {
        Self { autostart: false }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct DiskConfig {
    path: String,
    readonly: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Vm {
    pub name: String,
    pub vcpu: u64,
    pub vram: u64,
    pub uuid: Uuid,
    pub disk: Vec<DiskConfig>,
    pub config: VirshleVmConfig,
}
impl Default for Vm {
    fn default() -> Self {
        Self {
            name: template::random_name().unwrap(),
            vcpu: 1,
            vram: 2,
            uuid: Uuid::new_v4(),
            disk: vec![],
            config: Default::default(),
        }
    }
}

impl Vm {
    /*
     * Remove a vm and its definition.
     */
    pub fn delete(&mut self) {}
    /*
     * Saves the machine definition in the virshle directory at:
     * `/var/lib/virshle/vm/<vm_uuid>`
     * And persists vm "uuid" and "name" into the sqlite database at:
     * `/va/lib/virshle/virshle.sqlite`
     * for fast resource retrieving.
     *
     * ```rust
     * vm.save_definition()?;
     * ```
     *
     * You can find the definition by name and bring the vm
     * back up with:
     * ```rust
     * Vm::get("vm_name")?.set()?;
     * ```
     */
    pub async fn save_definition(&self) -> Result<(), VirshleError> {
        let res = toml::to_string(&self);
        let value: String = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlSerError(e);
                return Err(err.into());
            }
        };

        // Save definition to virshle managed file.
        let definition_path = MANAGED_DIR.to_owned() + "/vm/" + &self.uuid.to_string() + ".toml";
        let toml = toml::to_string(&value);
        let mut file = fs::File::create(definition_path)?;
        file.write_all(value.as_bytes())?;

        // Save Vm to db.
        let record = database::entity::vm::ActiveModel {
            uuid: ActiveValue::Set(self.uuid.to_string()),
            name: ActiveValue::Set(self.name.clone()),
            config: ActiveValue::Set(serde_json::to_value(&self.to_vmm_config()?)?),
            ..Default::default()
        };

        let db = connect_db().await?;
        database::prelude::Vm::insert(record).exec(&db).await?;

        Ok(())
    }

    /*
     * Bring the virtual machine up.
     */
    pub async fn create(&mut self) -> Result<Self, VirshleError> {
        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string();
        let cmd = format!(
            "cloud-hypervisor \
                --api-socket {socket}.sock \
            "
        );
        let mut proc = Process::new(&cmd);
        proc.run_detached()?;
        self.save_definition().await?;
        Ok(self.to_owned())
    }
    /*
     * Shut the virtual machine down.
     */
    pub async fn shutdown(&mut self) -> Result<(), VirshleError> {
        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string() + ".sock";
        let endpoint = "/api/v1/vm.info";
        let conn = Connection::open(&socket).await?;

        let response = conn.put::<Vm>(endpoint, None).await?;
        Ok(())
    }
    /*
     * Shut the virtual machine down.
     */
    pub async fn start(&mut self) -> Result<(), VirshleError> {
        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string() + ".sock";
        let endpoint = "/api/v1/vm.boot";
        let conn = Connection::open(&socket).await?;
        let response = conn.put::<Vm>(endpoint, None).await?;
        Ok(())
    }
    /*
     * Should be renamed to get_info();
     *
     */
    pub async fn update(&mut self) -> Result<&mut Vm, VirshleError> {
        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string() + ".sock";
        let endpoint = "/api/v1/vm.info";
        let conn = Connection::open(&socket).await?;

        let response = conn.get(endpoint).await?;

        let data = response.to_string().await?;
        println!("{:#?}", data);
        let data: VmInfoResponse = serde_json::from_str(&data)?;

        // self.state = Some(data.state);
        Ok(self)
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
    pub async fn get_state(&self) -> Result<VmState, VirshleError> {
        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string() + ".sock";
        let endpoint = "/api/v1/vm.info";

        let conn = Connection::open(&socket).await;
        let state: VmState = match conn {
            Ok(conn) => {
                let response = conn.get(endpoint).await?;
                let data = response.to_string().await?;
                let data: VmInfoResponse = serde_json::from_str(&data)?;
                VmState::from(data.state)
            }
            Err(e) => VmState::NotCreated,
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

    // #[tokio::test]
    async fn set_vm_from_file() -> Result<()> {
        // Get file
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../templates/ch/vm/xs.toml");
        let path = path.display().to_string();

        let mut item = Vm::from_file(&path)?;
        item.create().await?;
        Ok(())
    }

    #[tokio::test]
    async fn set_vm() -> Result<()> {
        let mut item = Vm::default();
        item.create().await?;
        Ok(())
    }
    // #[tokio::test]
    async fn delete_vm() -> Result<()> {
        let mut item = Vm::default();
        item.shutdown().await?;
        Ok(())
    }
}
