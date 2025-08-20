pub mod account;
pub mod create;
pub mod delete;
pub mod from;
pub mod getters;
pub mod init;
pub mod template;
pub mod to_vmm_types;

// Reexports
pub use account::Account;
pub use getters::VmInfo;
pub use init::{InitData, UserData, VmData};
pub use template::VmTemplate;

use crate::network::ip;
use to_vmm_types::VmConfig;

// Time
use chrono::{DateTime, NaiveDateTime, Utc};

use std::fs::File;
use std::os::fd::{AsFd, AsRawFd, RawFd};

// Serde
use convert_case::{Case, Casing};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, Value};
use std::fmt;

// Socket
use std::fs;
use std::os::unix::fs::PermissionsExt;
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

// Error Handling
use miette::{IntoDiagnostic, Result};
use tracing::{debug, error, info, trace};
use virshle_error::{LibError, VirshleError};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct VmConfigPlus {
    /// The account the vm is linked to.
    pub inner: Option<String>,

    // Unused
    pub autostart: bool,
}

impl VmConfigPlus {
    pub fn new<T>(inner: &T) -> Result<Self, VirshleError>
    where
        T: Serialize + DeserializeOwned + std::fmt::Debug,
    {
        let res = VmConfigPlus {
            inner: Some(serde_json::to_string(inner)?),
            ..Default::default()
        };
        Ok(res)
    }
}
impl Default for VmConfigPlus {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            autostart: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct VmNet {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: NetType,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NetType {
    Vhost(Vhost),
    Tap(Tap),
    // Can't pass macvtap through the Cloud-hypervisor http API.
    // Must be deprecated because actual ch implementation sucks!
    #[serde(rename = "macvtap")]
    MacVTap(Tap),
}
impl fmt::Display for NetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            NetType::Vhost(v) => "vhost".to_owned(),
            NetType::Tap(v) => "tap".to_owned(),
            NetType::MacVTap(v) => "macvtap".to_owned(),
        };
        write!(f, "{}", string)
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Tap {
    // Set static mac address or random if none.
    pub mac: Option<String>,
    // Request a static ipv4 ip on the interface.
    pub ip: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Vhost {
    // Set static mac address or random if none.
    pub mac: Option<String>,
    // Request a static ipv4 ip on the interface.
    pub ip: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Vm {
    // id from sqlite database
    pub id: Option<u64>,
    pub name: String,
    pub vcpu: u64,
    // vram in Gib
    pub vram: u64,
    pub net: Option<Vec<VmNet>>,
    pub uuid: Uuid,
    pub disk: Vec<Disk>,

    // Date
    #[serde(skip)]
    pub created_at: NaiveDateTime,
    #[serde(skip)]
    pub updated_at: NaiveDateTime,

    // Very optional vm parameters.
    /// Room for additional parameters (unused for now).
    pub config: Option<VmConfigPlus>,
}

impl Default for Vm {
    fn default() -> Self {
        let now: NaiveDateTime = Utc::now().naive_utc();

        Self {
            id: None,
            name: random_name().unwrap(),
            vcpu: 1,
            // vram in Gib
            vram: 2,
            net: None,
            uuid: Uuid::new_v4(),
            disk: vec![],

            // Date
            created_at: now,
            updated_at: now,

            config: Default::default(),
        }
    }
}

impl Vm {
    /// Start Vm
    #[tracing::instrument(skip_all)]
    pub async fn start(
        &mut self,
        user_data: Option<UserData>,
        attach: Option<bool>,
    ) -> Result<Vm, VirshleError> {
        // Create ressources
        self.add_init_disk(user_data)?;
        self.create_networks()?;

        self.start_vmm(attach).await?;
        self.push_config_to_vmm().await?;

        let mut conn = Connection::from(&self.clone());
        let mut rest = RestClient::from(&mut conn);

        let endpoint = "/api/v1/vm.boot";
        let response = rest.put::<()>(endpoint, None).await?;

        if response.status().is_success() {
            let msg = &response.to_string().await?;
            trace!("{}", &msg);
        } else {
            let err_msg = &response.to_string().await?;
            error!("{}", &err_msg);

            let message = "Couldn't boot vm.";
            return Err(LibError::builder()
                .msg(&message)
                .help(&err_msg)
                .build()
                .into());
        }
        self.set_vsock_permissions().await?;

        info!("started vm {:#?}", self.name);
        Ok(self.to_owned())
    }
    /// Start or Restart a VMM.
    async fn start_vmm(&self, attach: Option<bool>) -> Result<(), VirshleError> {
        // Safeguard: remove old process and artifacts
        self.delete_ch_proc()?;

        #[cfg(debug_assertions)]
        let mut cmd = format!("cloud-hypervisor");
        #[cfg(not(debug_assertions))]
        let mut cmd = format!("cloud-hypervisor");

        // If we can't establish connection to socket,
        // this means cloud-hypervisor is dead.
        // So we start a new viable process.
        let mut conn = Connection::from(self);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/vmm.ping");

        if rest.open().await.is_err() || rest.ping().await.is_err() {
            match attach {
                Some(true) => {
                    cmd = format!(
                        "kitty \
                            --title ttyS0@vm-{} \
                            --hold sh -c \"{} --api-socket {}\"",
                        &self.name,
                        cmd,
                        &self.get_socket()?
                    );
                    Process::new()
                        .stdin(&cmd)
                        .term()
                        .background()
                        .detach()
                        .run()?;
                    info!("launching: {:#?}", &cmd);
                }
                _ => {
                    cmd = format!("{} --api-socket {}", &cmd, &self.get_socket()?);
                    Process::new()
                        .stdin(&cmd)
                        .orphan()
                        .background()
                        .detach()
                        .run()?;
                    info!("launching: {:#?}", &cmd);
                }
            };

            // Wait until socket is created
            let socket = &self.get_socket()?;
            let path = Path::new(socket);
            while !path.exists() {
                tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
            }

            // Set loose permission on cloud-hypervisor socket.
            #[cfg(not(debug_assertions))]
            {
                let mut perms = fs::metadata(&path)?.permissions();
                perms.set_mode(0o774);
                fs::set_permissions(&path, perms)?;
            }
            #[cfg(debug_assertions)]
            Process::new()
                .stdin(&format!("sudo chmod 774 {}", &socket))
                .run()?;
        }
        Ok(())
    }

    /// Shut the virtual machine down and removes artifacts.
    /// Should silently fail when vm is already down.
    #[tracing::instrument(skip_all)]
    pub async fn shutdown(&self) -> Result<Self, VirshleError> {
        let mut conn = Connection::from(self);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.open().await?;
        rest.ping().await?;

        // Soft shutdown VM.
        let endpoint = "/vm.shutdown";
        let response = rest.put::<()>(endpoint, None).await?;

        // Soft shutdown vmm.
        let endpoint = "/vmm.shutdown";
        let response = rest.put::<()>(endpoint, None).await?;

        // Remove ch process
        self.delete_ch_proc()?;
        // Remove network ports
        self.delete_networks()?;

        info!("stopped vm {}", self.name);
        Ok(self.to_owned())
    }

    #[tracing::instrument(skip_all)]
    pub async fn pause(&self) -> Result<(), VirshleError> {
        let mut conn = Connection::from(self);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.open().await?;
        rest.ping().await?;

        let endpoint = "/vm.pause";
        let response = rest.put::<()>(endpoint, None).await?;

        info!("paused vm {}", self.name);
        Ok(())
    }

    /// Bring the virtual machine up.
    async fn push_config_to_vmm(&self) -> Result<(), VirshleError> {
        let config = VmConfig::from(self).await?;
        trace!("{:#?}", config);

        let mut conn = Connection::from(self);
        let mut rest = RestClient::from(&mut conn);

        let endpoint = "/api/v1/vm.create";
        let response = rest.put::<VmConfig>(endpoint, Some(config)).await?;

        if response.status().is_success() {
            let msg = &response.to_string().await?;
            trace!("{}", &msg);
        } else {
            let err_msg = &response.to_string().await?;
            error!("{}", &err_msg);
        }

        Ok(())
    }
    /// Widden vsock permissions to allow ssh connection from the owning group.
    ///
    /// Socket is not created on ch proc start, but after vm boot.
    /// So this function is to be used after vm boot.
    async fn set_vsock_permissions(&self) -> Result<(), VirshleError> {
        // Wait until socket is created
        let socket = self.get_vsocket()?;
        let path = Path::new(&socket);
        while !path.exists() {
            tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
        }

        // Set loose permission on vsocket.
        #[cfg(not(debug_assertions))]
        {
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o774);
            fs::set_permissions(&path, perms)?;
        }
        #[cfg(debug_assertions)]
        Process::new()
            .stdin(&format!("sudo chmod 774 {}", &socket))
            .run()?;
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
        item.create(None).await?;
        Ok(())
    }

    #[tokio::test]
    async fn set_vm() -> Result<()> {
        let mut item = Vm::default();
        item.create(None).await?;
        Ok(())
    }
    // #[tokio::test]
    async fn delete_vm() -> Result<()> {
        let item = Vm::default();
        item.shutdown().await?;
        Ok(())
    }
}
