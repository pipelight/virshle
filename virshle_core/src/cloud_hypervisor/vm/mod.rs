pub mod from;

pub mod init;

pub mod crud;
pub mod getters;

pub use crate::cloud_hypervisor::VmState;
pub use from::VmTemplate;

use super::vmm_types::VmConfig;

use std::fmt;

// Ovs
use crate::network::Ovs;

use convert_case::{Case, Casing};
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
pub struct VmNet {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: NetType,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NetType {
    Vhost(Vhost),
}
impl fmt::Display for NetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            NetType::Vhost(v) => "vhost".to_owned(),
        };
        write!(f, "{}", string)
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Vhost {
    // Set static mac address or random if none.
    pub mac: Option<String>,
    // Request a static ipv4 ip on the interface.
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
    async fn connection(&self) -> Result<Connection, VirshleError> {
        let socket = self.get_socket()?;
        Connection::open(&socket).await
    }

    /*
     * Start or Restart a Vm
     */
    async fn start_vmm(&self) -> Result<(), VirshleError> {
        // Safeguard: remove old process and artifacts
        self.delete_ch_proc()?;

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

    /*
     * Shut the virtual machine down.
     */
    pub async fn shutdown(&self) -> Result<(), VirshleError> {
        let socket = &self.get_socket()?;
        let endpoint = "/api/v1/vm.shutdown";
        let conn = Connection::open(&socket).await?;

        let response = conn.put::<()>(endpoint, None).await?;
        Ok(())
    }
    /*
     * Start the virtual machine.
     */
    pub async fn start(&mut self) -> Result<(), VirshleError> {
        self.create_networks()?;

        self.start_vmm().await?;
        self.push_config_to_vmm().await?;

        let socket = &self.get_socket()?;
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

        let socket = &self.get_socket()?;
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
