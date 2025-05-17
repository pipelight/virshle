pub mod crud;
pub mod from;
pub mod getters;
pub mod init;

// Reexports
pub use from::VmTemplate;

use super::vmm_types::VmConfig;

use std::fmt;

use serde::{Deserialize, Serialize};
use std::path::Path;

use pipelight_exec::Process;
use std::io::Write;
use std::process::{Command, Stdio};

use super::disk::Disk;
use super::rand::random_name;
use uuid::Uuid;

// Http
use crate::connection::{Connection, ConnectionHandle, UnixConnection, VmConnection};
use crate::http_request::HttpRequest;

//Database
use crate::database::entity::{prelude::*, *};

// Error Handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Account {
    uuid: String,
    name: String,
}
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct VmConfigPlus {
    /// The account the vm is linked to.
    pub owner: Option<Account>,

    // Unused
    pub autostart: bool,
    pub attach: bool,
}

impl Default for VmConfigPlus {
    fn default() -> Self {
        Self {
            owner: Default::default(),
            autostart: false,
            attach: false,
        }
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

    // Very optional vm parameters.
    /// Room for additional parameters (unused for now).
    pub config: Option<VmConfigPlus>,
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
    async fn connection(&self) -> Result<VmConnection, VirshleError> {
        let socket = &self.get_socket()?;
        let mut conn = VmConnection(Connection::UnixConnection(UnixConnection::new(socket)));
        conn.open().await?;
        Ok(conn)
    }

    /*
     * Start or Restart a Vm
     */
    async fn start_vmm(&self) -> Result<(), VirshleError> {
        // Safeguard: remove old process and artifacts
        self.delete_ch_proc()?;

        // If can't establish connection to socket,
        // Then start new process.
        if self.connection().await.is_err() {
            if let Some(config) = &self.config {
                if config.attach {
                    let proc = Command::new("cloud-hypervisor")
                        .args(["--api-socket", &self.get_socket()?])
                        .spawn()?;
                }
            } else {
                let cmd = format!("cloud-hypervisor --api-socket {}", &self.get_socket()?);
                let mut proc = Process::new();
                proc.stdin(&cmd).background().detach().run()?;
            }
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
        let mut conn = self.connection().await?;

        let response = conn.put::<()>(endpoint, None).await?;
        Ok(())
    }

    pub fn attach(&mut self) -> Result<&mut Self, VirshleError> {
        match &mut self.config {
            Some(v) => v.attach = true,
            None => {
                let mut config = VmConfigPlus::default();
                config.attach = true;
                self.config = Some(config);
            }
        }
        Ok(self)
    }
    /*
     * Create needed resources (network)
     * And start the virtual machine and .
     */
    pub async fn start(&mut self) -> Result<(), VirshleError> {
        self.create_networks()?;

        self.start_vmm().await?;

        // Provision with user defined data
        self.add_init_disk()?;

        self.push_config_to_vmm().await?;

        let socket = &self.get_socket()?;
        let endpoint = "/api/v1/vm.boot";
        let mut conn = self.connection().await?;
        let response = conn.put::<()>(endpoint, None).await?;

        if !response.status().is_success() {
            let message = "Couldn't create vm.";
            return Err(LibError::builder()
                .msg(&message)
                .help(&response.to_string().await?)
                .build()
                .into());
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
        let mut conn = self.connection().await?;

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
