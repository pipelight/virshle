mod info;
pub mod utils;
pub use info::DiskInfo;

// Struct
use super::vm::{InitData, UserData, Vm, VmData};

// Filesystem
// use tokio::fs::{self, File};
// use tokio::io::AsyncWrite;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;

// Process
use pipelight_exec::{Process, Status};

// Cloud Hypervisor
use uuid::Uuid;

use serde::{Deserialize, Serialize};

// Error Handling
use log::{debug, error, info, trace};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct DiskTemplate {
    pub name: String,
    pub path: String,
    pub readonly: Option<bool>,
}
impl DiskTemplate {
    pub fn get_size(&self) -> Result<u64, VirshleError> {
        let source = utils::shellexpand(&self.path)?;
        let path = Path::new(&source);
        if path.exists() && path.is_file() {
            let metadata = std::fs::metadata(path)?;
            let size = metadata.len();
            Ok(size)
        } else {
            Err(LibError::builder()
                .msg("Counldn't get disk file size.")
                .help("Disk doesn't exist or is unreachable")
                .build()
                .into())
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Disk {
    pub name: String,
    pub path: String,
    pub readonly: Option<bool>,
}
impl Disk {
    pub fn get_size(&self) -> Result<u64, VirshleError> {
        let path = Path::new(&self.path);
        if path.exists() && path.is_file() {
            let metadata = std::fs::metadata(path)?;
            let size = metadata.len();
            Ok(size)
        } else {
            Err(LibError::builder()
                .msg("Counldn't get disk file size.")
                .help("Disk doesn't exist or is unreachable")
                .build()
                .into())
        }
    }
}

impl From<&DiskTemplate> for Disk {
    fn from(e: &DiskTemplate) -> Self {
        Self {
            name: e.name.to_owned(),
            path: e.path.to_owned(),
            readonly: e.readonly,
        }
    }
}

/*
* An ephemeral disk that is mounted/unmounted to vm on boot.
* to provision with custom user datas.
*/
#[derive(Debug, Eq, PartialEq)]
pub struct InitDisk<'a> {
    pub vm: &'a Vm,
}

impl<'a> From<&'a InitDisk<'a>> for Disk {
    fn from(e: &InitDisk) -> Self {
        let disk_dir = e.vm.get_disks_dir().unwrap();
        let path = format!("{disk_dir}/pipelight-init");
        Self {
            name: "init".to_owned(),
            path,
            ..Default::default() // readonly: Some(true),
        }
    }
}

impl InitDisk<'_> {
    /*
     * Write pipelight configuration file to init disk.
     */
    pub fn write_init_files(&self, init_data: &InitData) -> Result<&Self, VirshleError> {
        let mount_dir = self.vm.get_mount_dir()?;
        let target = format!("{mount_dir}/pipelight-init/pipelight.toml");

        // Remove old pipeline
        let path = Path::new(&target);
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        // Write file to disk
        let p_config = init_data.to_pipelight_toml_config()?;
        let bytes = p_config.as_bytes();
        let mut file = std::fs::File::create(path)?;
        file.write_all(bytes)?;

        Ok(self)
    }
    /*
     * Create an init disk on host filesystem.
     */
    pub fn create(&self) -> Result<&Self, VirshleError> {
        let disk_dir = self.vm.get_disks_dir()?;
        let source = format!("{disk_dir}/pipelight-init");
        utils::make_empty_file(&source)?;
        utils::format_to_vfat(&source)?;
        Ok(self)
    }

    pub fn mount(&self) -> Result<&Self, VirshleError> {
        let disk_dir = self.vm.get_disks_dir()?;
        let source = format!("{disk_dir}/pipelight-init");

        let mount_dir = self.vm.get_mount_dir()?;
        let target = format!("{mount_dir}/pipelight-init");

        #[cfg(debug_assertions)]
        utils::mount(&source, &target)?;
        #[cfg(not(debug_assertions))]
        utils::_mount(&source, &target)?;

        Ok(self)
    }
    pub fn umount(&self) -> Result<&Self, VirshleError> {
        let mount_dir = self.vm.get_mount_dir()?;
        let target = format!("{mount_dir}/pipelight-init");

        // #[cfg(debug_assertions)]
        utils::umount(&target)?;
        // #[cfg(not(debug_assertions))]
        // utils::_umount(&&target)?;
        Ok(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_init_disk() -> Result<()> {
        // let vm = Vm::default();
        let vms = Vm::get_all().await?;
        let mut vm = vms.first().unwrap().to_owned();

        let init_disk = InitDisk { vm: &mut vm };
        let init_data = InitData::default();
        init_disk
            .create()?
            .mount()?
            .write_init_files(&init_data)?
            .umount()?;

        // println!("{:#?}", res);
        Ok(())
    }
}
