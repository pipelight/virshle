pub mod create;
pub mod delete;
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
use crate::connection::{Connection, ConnectionHandle, UnixConnection};
use crate::http_request::{Rest, RestClient};

//Database
use crate::database::entity::{prelude::*, *};

// Error Handling
use log::{debug, error, info, trace};
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
    Tap(Tap),
}
impl fmt::Display for NetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            NetType::Vhost(v) => "vhost".to_owned(),
            NetType::Tap(v) => "tap".to_owned(),
        };
        write!(f, "{}", string)
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Tap {
    // Set static mac address or random if none.
    pub mac: Option<String>,
    // Request a static ipv4 ip on the interface.
    pub ip: Option<String>,
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
    /*
     * Start or Restart a Vm
     */
    async fn start_vmm(&self) -> Result<(), VirshleError> {
        // Safeguard: remove old process and artifacts
        self.delete_ch_proc()?;

        #[cfg(debug_assertions)]
        let cmd = format!("sudo cloud-hypervisor");
        #[cfg(not(debug_assertions))]
        let cmd = format!("cloud-hypervisor");

        // If we can't establish connection to socket,
        // this means cloud-hypervisor is dead.
        // So we start a new viable process.
        let mut conn = Connection::from(self);
        if conn.open().await.is_err() {
            match self.is_attach().ok() {
                Some(true) => {
                    let cmd = format!(
                        "kitty --hold sh -c \"{} --api-socket {}\"",
                        cmd,
                        &self.get_socket()?
                    );
                    Process::new()
                        .stdin(&cmd)
                        .term()
                        .background()
                        .detach()
                        .run()?;
                    info!("Launching: {}", &cmd);
                }
                _ => {
                    let cmd = format!("{} --api-socket {}", &cmd, &self.get_socket()?);
                    Process::new()
                        .stdin(&cmd)
                        .orphan()
                        .background()
                        .detach()
                        .run()?;
                }
            };

            // Wait until socket is created
            let socket = &self.get_socket()?;
            let path = Path::new(socket);
            while !path.exists() {
                tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
            }

            // Set loose permission on cloud-hypervisor socket.
            #[cfg(debug_assertions)]
            Process::new()
                .stdin(&format!("sudo chgrp users {}", &self.get_socket()?))
                .run()?;
            #[cfg(debug_assertions)]
            Process::new()
                .stdin(&format!("sudo chmod 774 {}", &self.get_socket()?))
                .run()?;

            debug!("Started vm: {:#?}", cmd);
        }
        Ok(())
    }

    /*
     * Shut the virtual machine down.
     */
    pub async fn shutdown(&self) -> Result<(), VirshleError> {
        let socket = &self.get_socket()?;
        let endpoint = "/api/v1/vm.shutdown";
        let mut conn = Connection::from(self);

        let mut rest = RestClient::from(&mut conn);
        let response = rest.put::<()>(endpoint, None).await?;

        // Remove ch process
        self.delete_ch_proc()?;

        // Remove network ports
        self.delete_networks();

        Ok(())
    }
    pub async fn pause(&self) -> Result<(), VirshleError> {
        let socket = &self.get_socket()?;
        let endpoint = "/api/v1/vm.pause";
        let mut conn = Connection::from(self);

        let mut rest = RestClient::from(&mut conn);
        let response = rest.put::<()>(endpoint, None).await?;
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
     * Bring the virtual machine up.
     */
    async fn push_config_to_vmm(&self) -> Result<(), VirshleError> {
        let config = VmConfig::from(self);
        trace!("{:#?}", config);

        let mut conn = Connection::from(self);
        let mut rest = RestClient::from(&mut conn);

        let endpoint = "/api/v1/vm.create";
        let response = rest.put::<VmConfig>(endpoint, Some(config)).await?;

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
