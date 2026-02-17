mod display;
pub mod from;
pub mod getters;
pub mod init;

// High level methods to orchestrate VMs.
pub mod crud;

// Methods
// Lower level methods for:
// - cloud hypervisor API.
// - database operations.
// - host network manipulation.
pub mod database;
pub mod networks;

// Reexports
pub use crate::config::{DiskTemplate, UserData, VmNet};
pub use display::VmTable;
pub use getters::VmInfo;
pub use init::{InitData, VmData};

// Time
use chrono::{NaiveDateTime, Utc};

// Serde
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

// Socket
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::disk::Disk;
use super::rand::random_name;
use uuid::Uuid;

use pipelight_exec::Process;

// Error Handling
use miette::Result;
use virshle_error::VirshleError;

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
pub struct Vm {
    // id from sqlite database
    pub id: Option<u64>,
    pub name: String,
    pub vcpu: u64,
    pub vram: String,
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
            vram: "1GiB".to_owned(),
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
