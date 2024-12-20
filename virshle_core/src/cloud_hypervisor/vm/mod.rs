pub mod from;
pub mod getters;
pub mod to;

pub use from::VmTemplate;
pub use getters::VmState;

// Cloud Hypervisor
use uuid::Uuid;
use vmm::api::VmInfoResponse;
use vmm::{
    vm::VmState as ChVmState,
    vm_config::{
        // defaults
        default_console,
        default_netconfig_mac,

        default_serial,
        CpusConfig,
        DiskConfig as ChDiskConfig,
        MemoryConfig,
        NetConfig,
        RngConfig,
        VmConfig,
    },
};

use hyper::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use std::path::Path;

use pipelight_exec::{Finder, Process};
use std::io::Write;
use tabled::{Table, Tabled};

use super::disk::Disk;
use super::net::{Ip, Net};
use super::rand::random_name;

// Http
use crate::http_cli::Connection;

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::{prelude::*, *};
use sea_orm::{
    prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult, IntoActiveModel,
};

use crate::config::MANAGED_DIR;
use crate::display::vm::*;

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};
use virshle_error::{LibError, VirshleError};

use serde_json::{from_slice, Value};
use tokio::fs;

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
#[serde(rename_all = "snake_case")]
pub enum VmNet {
    Tap(Tap),
    Bridge(Bridge),
}
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Bridge {
    // Bridge name
    pub name: String,
    // Request a static ip on the interface.
    pub ip: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Tap {
    // Tap interface name
    pub name: Option<String>,
    // Request a static ip on the interface.
    pub ip: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Vm {
    pub name: String,
    pub vcpu: u64,
    // vram in Mib
    pub vram: u64,
    pub net: Option<Vec<VmNet>>,
    pub uuid: Uuid,
    pub disk: Vec<Disk>,
    pub config: Option<VirshleVmConfig>,
}
impl Default for Vm {
    fn default() -> Self {
        Self {
            name: random_name().unwrap(),
            vcpu: 1,
            // vram in Mib
            vram: 2,
            net: None,
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
    pub async fn delete(&self) -> Result<Self, VirshleError> {
        // Remove running processes
        Finder::new()
            .seed("cloud-hypervisor")
            .seed(&self.uuid.to_string())
            .search_no_parents()?
            .kill()?;

        // Purge disk/net

        // Clean existing socket
        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string() + ".sock";
        let path = Path::new(&socket);
        if path.exists() {
            fs::remove_file(&socket).await?;
        }

        // Remove record from database
        let db = connect_db().await?;
        let record = database::prelude::Vm::find()
            .filter(database::entity::vm::Column::Name.eq(&self.name))
            .one(&db)
            .await?;
        if let Some(record) = record {
            database::prelude::Vm::delete(record.into_active_model())
                .exec(&db)
                .await?;
        }
        Ok(self.to_owned())
    }
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
    async fn save_definition(&self) -> Result<(), VirshleError> {
        let res = toml::to_string(&self);
        let value: String = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlSerError(e);
                return Err(err.into());
            }
        };

        // Save Vm to db.
        let record = database::entity::vm::ActiveModel {
            uuid: ActiveValue::Set(self.uuid.to_string()),
            name: ActiveValue::Set(self.name.clone()),
            definition: ActiveValue::Set(serde_json::to_value(&self)?),
            ..Default::default()
        };

        let db = connect_db().await?;
        database::prelude::Vm::insert(record).exec(&db).await?;

        Ok(())
    }

    /*
     * Start networks and link to vm definition
     */
    async fn start_networks(&mut self) -> Result<(), VirshleError> {
        if let Some(nets) = &self.net {
            for net in nets {
                match net {
                    VmNet::Tap(v) => {
                        let host_net = Ip::get_default_interface_name()?;
                        let cmd = format!(
                            "sudo ip link add {} name {} type macvtap",
                            host_net, self.name
                        );
                        let mut proc = Process::new(&cmd);
                        proc.run_piped()?;

                        let mac = default_netconfig_mac();
                        let cmd = format!("sudo ip link set {} address {} up", self.name, mac);
                        let mut proc = Process::new(&cmd);
                        proc.run_piped()?;
                    }
                    VmNet::Bridge(v) => {}
                }
            }
        }
        Ok(())
    }

    async fn start_vmm(&self) -> Result<(), VirshleError> {
        // Remove running vmm processes
        Finder::new()
            .seed("cloud-hypervisor")
            .seed(&self.uuid.to_string())
            .search_no_parents()?
            .kill()?;

        // Remove existing socket
        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string() + ".sock";
        let path = Path::new(&socket);
        if path.exists() {
            fs::remove_file(&socket).await?;
        }

        match Connection::open(&socket).await {
            Ok(_) => Ok(()),
            Err(_) => {
                let cmd = format!("cloud-hypervisor --api-socket {socket}");
                let mut proc = Process::new(&cmd);
                proc.run_detached()?;

                // Wait until socket is created
                while !path.exists() {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }

                Ok(())
            }
        }
    }

    pub async fn create(&self) -> Result<Self, VirshleError> {
        self.save_definition().await?;
        Ok(self.to_owned())
    }
    /*
     * Shut the virtual machine down.
     */
    pub async fn shutdown(&self) -> Result<(), VirshleError> {
        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string() + ".sock";
        let endpoint = "/api/v1/vm.shutdown";
        let conn = Connection::open(&socket).await?;

        let response = conn.put::<()>(endpoint, None).await?;
        Ok(())
    }
    /*
     * Shut the virtual machine down.
     */
    pub async fn start(&mut self) -> Result<(), VirshleError> {
        self.start_networks().await?;
        self.start_vmm().await?;
        self.push_config_to_vmm().await?;

        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string() + ".sock";
        let endpoint = "/api/v1/vm.boot";
        let conn = Connection::open(&socket).await?;
        let response = conn.put::<()>(endpoint, None).await?;
        Ok(())
    }
    /*
     * Bring the virtual machine up.
     */
    async fn push_config_to_vmm(&self) -> Result<(), VirshleError> {
        let config = self.to_vmm_config()?;

        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string() + ".sock";
        let endpoint = "/api/v1/vm.create";
        let conn = Connection::open(&socket).await?;

        let response = conn.put::<VmConfig>(endpoint, Some(config)).await?;
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

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
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
