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
    /*
     * Create and provision an init disk,
     * and add it vm config.
     */
    pub fn add_init_disk(&mut self) -> Result<&Self, VirshleError> {
        // Make disk
        let init_disk = InitDisk { vm: self };
        init_disk.create()?.mount()?.write_init_files()?.umount()?;

        // Add to vm config
        let disk = Disk::from(&init_disk);
        self.disk.push(disk);

        Ok(self)
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

        vm.add_init_disk()?;
        Ok(())
    }
}
