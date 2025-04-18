use super::Vm;
use crate::cloud_hypervisor::{Disk, InitDisk};

use std::path::Path;
use sys_mount::{Mount, MountFlags, SupportedFilesystems, Unmount, UnmountFlags};

// Global
use crate::config::MANAGED_DIR;

// "mkdir -p ./scripts/mnt/pipelight-init",
// "mount -t ext4 -o loop ./scripts/pipelight-init.img ./scripts/mnt/pipelight-init",
// "cp -r /pipelight-init/.* ./scripts/mnt/pipelight-init",
// "cp -r /pipelight-init/* ./scripts/mnt/pipelight-init",
// "umount ./scripts/mnt/pipelight-init",

use pipelight_exec::{Process, Status};
use std::fs;

// Error handling
use log::info;
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError};

impl Vm {
    pub fn create_init_disk(&mut self) -> Result<InitDisk, VirshleError> {
        let init_disk = InitDisk { vm: self };

        init_disk.create()?.mount()?;
        init_disk.umount()?;

        Ok(init_disk)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_init_disk_creation() -> Result<()> {
        // let vm = Vm::default();
        let vms = Vm::get_all().await?;
        let mut vm = vms.first().unwrap().to_owned();
        // println!("{:#?}", &vm);

        vm.create_init_disk()?;
        Ok(())
    }
}
