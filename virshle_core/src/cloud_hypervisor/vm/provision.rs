use super::{Disk, Vm};
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

pub fn try_mount_vfat(source: &str, target: &str) -> Result<(), VirshleError> {
    // Ensure mounting directory exists
    fs::create_dir_all(&target)?;

    let mut commands = vec![];

    // TODO(): add systemd unit mountcap.
    // Mount need root priviledge
    #[cfg(debug_assertions)]
    commands.push(format!("sudo mount -t vfat -o loop {source} {target}"));
    #[cfg(not(debug_assertions))]
    commands.push(format!("mount -t vfat -o loop {source} {target}"));

    for cmd in commands {
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        match res.state.status {
            Some(Status::Failed) => {
                let message = format!("Couldn't mount vfat disk.");
                let help = res.io.stderr.unwrap_or_default();
                return Err(LibError::new(&message, &help).into());
            }
            _ => {
                info!("{:#?}", res.io.stdout);
            }
        };
    }
    Ok(())
}
pub fn try_umount(target: &str) -> Result<(), VirshleError> {
    let mut commands = vec![];

    // Umount need root priviledge
    #[cfg(debug_assertions)]
    commands.push(format!("sudo umount {target}"));
    #[cfg(not(debug_assertions))]
    commands.push(format!("umount {target}"));

    for cmd in commands {
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        match res.state.status {
            Some(Status::Failed) => {
                let message = format!("Couldn't unmount disk.");
                let help = res.io.stderr.unwrap_or_default();
                return Err(LibError::new(&message, &help).into());
            }
            _ => {
                info!("{:#?}", res.io.stdout);
            }
        };
    }

    // Clean mount points
    fs::remove_dir_all(&target)?;

    Ok(())
}

impl Vm {
    pub fn create_provision(&self) -> Result<(), VirshleError> {
        let disk_path = self.get_disk_dir()?;
        let mount_path = self.get_mount_dir()?;

        // Create a fresh pipelight-init disk.
        let source = format!("{disk_path}/pipelight-init");
        let commands = vec![
            format!("dd if=/dev/null of={source} bs=1M seek=10"),
            format!("mkfs.vfat -F 32 -n INIT {source}"),
        ];
        for cmd in commands {
            let mut proc = Process::new();
            let res = proc.stdin(&cmd).run()?;

            match res.state.status {
                Some(Status::Failed) => {
                    let message = format!("Couldn't create pipelight-init disk.");
                    let help = res.io.stderr.unwrap_or_default();
                    return Err(LibError::new(&message, &help).into());
                }
                _ => {
                    info!("{:#?}", res.io.stdout);
                }
            };
        }

        // Ensure mount directory exists and unmount old disk if already mounted.
        let target = format!("{mount_path}/pipelight-init");
        let path = Path::new(&target);
        if path.exists() {
            try_umount(&target).ok();
        }

        // Mount and add files to disk
        try_mount_vfat(&source, &target)?;

        // Unmount device
        // try_umount(&target)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_init_disk_creation() -> Result<()> {
        // let vm = Vm::default();
        let vms = Vm::get_all().await?;
        let vm = vms.first().unwrap();
        // println!("{:#?}", &vm);

        vm.create_provision()?;
        Ok(())
    }
}
