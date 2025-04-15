pub mod from;
pub mod getters;

pub use crate::cloud_hypervisor::VmState;
pub use from::VmTemplate;

use super::vmm_types::VmConfig;

// Ovs
use crate::network::Ovs;
// Network socket
use std::os::unix::net::{SocketAddr, UnixListener, UnixStream};

use uuid::Uuid;

use hyper::{Request, StatusCode};
use serde::{Deserialize, Serialize};
use std::path::Path;

use pipelight_exec::{Finder, Process};
use std::io::Write;
use tabled::{Table, Tabled};

use super::disk::Disk;
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
use virshle_error::{CastError, LibError, VirshleError};

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
    Vhost(Vhost),
}
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Vhost {
    // Set static mac address or random if none.
    pub mac: Option<String>,
    // Request a static ip on the interface.
    pub ip: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Vm {
    // id from sqlite database
    pub id: Option<u64>,
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
            id: None,
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
     * Remove a vm definition from database.
     * And delete vm ressources and process.
     */
    pub async fn delete(&self) -> Result<Self, VirshleError> {
        // Remove process and artifacts.
        self._clean_vmm().await?;
        // Remove disks from filesystem
        for disk in &self.disk {
            let path = Path::new(&disk.path);
            if path.exists() {
                fs::remove_file(&disk.path).await?;
            }
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
     * Persists vm into the sqlite database at: `/var/lib/virshle/virshle.sqlite`
     *
     * ```rust
     * vm.save_definition()?;
     * ```
     *
     * You can find the definition by name and bring the vm
     * back up with:
     *
     * ```rust
     * Vm::get("vm_name")?.start()?;
     * ```
     */
    async fn save_definition(&mut self) -> Result<(), VirshleError> {
        // Save Vm to db.
        let record = database::entity::vm::ActiveModel {
            uuid: ActiveValue::Set(self.uuid.to_string()),
            name: ActiveValue::Set(self.name.clone()),
            definition: ActiveValue::Set(serde_json::to_value(&self)?),
            ..Default::default()
        };

        let db = connect_db().await?;
        let res: InsertResult<vm::ActiveModel> =
            database::prelude::Vm::insert(record).exec(&db).await?;
        self.id = Some(res.last_insert_id as u64);

        Ok(())
    }

    async fn connection(&self) -> Result<Connection, VirshleError> {
        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string() + ".sock";
        Connection::open(&socket).await
    }

    /*
     * Delete the associated cloud-hypervisor running process,
     * and remove process artifacts (socket).
     */
    async fn _clean_vmm(&self) -> Result<(), VirshleError> {
        // Remove running vm hypervisor process
        Finder::new()
            .seed("cloud-hypervisor")
            .seed(&self.uuid.to_string())
            .search_no_parents()?
            .kill()?;

        // Remove existing ch api socket
        let socket = &self.get_socket()?;
        let path = Path::new(&socket);
        if path.exists() {
            fs::remove_file(&socket).await?;
        }
        // Remove existing ovs network socket
        let socket = &self.get_net_socket()?;
        let path = Path::new(&socket);
        if path.exists() {
            fs::remove_file(&socket).await?;
        }
        Ok(())
    }

    /*
     * Start or Restart a Vm
     */
    async fn start_vmm(&self) -> Result<(), VirshleError> {
        // Safeguard: remove old process and artifacts
        self._clean_vmm().await?;

        // If connection doesn't exist
        if self.connection().await.is_err() {
            let cmd = format!("cloud-hypervisor --api-socket {}", &self.get_socket()?);
            let mut proc = Process::new();
            proc.stdin(&cmd).background().detach().run()?;

            // Wait until socket is created
            let socket = &self.get_socket()?;
            let path = Path::new(socket);
            while !path.exists() {
                tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
            }
        }
        Ok(())
    }

    // pub async fn create_network(&mut self) -> Result<Self, VirshleError> {
    // }

    pub async fn create(&mut self) -> Result<Self, VirshleError> {
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
     * Start the virtual machine.
     */
    pub async fn start(&mut self) -> Result<(), VirshleError> {
        Ovs::create_vm_socket(&self)?;

        self.start_vmm().await?;
        self.push_config_to_vmm().await?;

        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string() + ".sock";
        let endpoint = "/api/v1/vm.boot";
        let conn = Connection::open(&socket).await?;
        let response = conn.put::<()>(endpoint, None).await?;

        if !response.status().is_success() {
            let message = "Couldn't create vm.";
            return Err(LibError::new(&message, &response.to_string().await?).into());
        }

        Ok(())
    }

    /*
     * Bring the virtual machine up.
     */
    async fn push_config_to_vmm(&self) -> Result<(), VirshleError> {
        let config = VmConfig::from(self);

        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string() + ".sock";
        let endpoint = "/api/v1/vm.create";
        let conn = Connection::open(&socket).await?;

        let response = conn.put::<VmConfig>(endpoint, Some(config)).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

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
