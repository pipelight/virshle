mod info;
mod utils;
pub use info::DiskInfo;
pub use utils::reverse_human_bytes;

// Struct
use super::vm::{InitData, UserData, Vm, VmData};

// Filesystem
use std::fs;
use std::io::Write;
use std::path::Path;

// Process
use pipelight_exec::{Process, Status};

// Cloud Hypervisor
use uuid::Uuid;

use serde::{Deserialize, Serialize};

// Error Handling
use log::{debug, info, trace};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct DiskTemplate {
    pub name: String,
    pub path: String,
    pub readonly: Option<bool>,
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
            let metadata = fs::metadata(path)?;
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
impl DiskTemplate {
    pub fn get_size(&self) -> Result<u64, VirshleError> {
        let source = shellexpand::tilde(&self.path).to_string();
        let path = Path::new(&source);
        if path.exists() && path.is_file() {
            let metadata = fs::metadata(path)?;
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
        let disk_dir = e.vm.get_disk_dir().unwrap();
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
            fs::remove_file(path)?;
        }
        // Write file to disk
        let p_config = init_data.to_pipelight_toml_config()?;
        let bytes = p_config.as_bytes();
        let mut file = fs::File::create(path)?;
        file.write_all(bytes)?;

        Ok(self)
    }
    /*
     * Create an init disk on host filesystem.
     */
    pub fn create(&self) -> Result<&Self, VirshleError> {
        let disk_dir = self.vm.get_disk_dir()?;
        let source = format!("{disk_dir}/pipelight-init");

        let mount_dir = self.vm.get_mount_dir()?;
        let target = format!("{mount_dir}/pipelight-init");

        // Create a fresh pipelight-init disk.
        let mut commands = vec![
            format!("dd if=/dev/null of={source} bs=1M seek=10"),
            format!("mkfs.vfat -F 32 -n INIT {source}"),
        ];

        #[cfg(debug_assertions)]
        commands.push(format!("sudo chmod 766 {source}"));
        #[cfg(not(debug_assertions))]
        commands.push(format!("chmod o+w {source}"));

        for cmd in commands {
            let mut proc = Process::new();
            let res = proc.stdin(&cmd).run()?;

            match res.state.status {
                Some(Status::Failed) => {
                    let message = format!("Couldn't create an init disk.");
                    let help = res.io.stderr.unwrap_or_default();
                    return Err(LibError::builder().msg(&message).help(&help).build().into());
                }
                _ => {
                    trace!(
                        "Executed {:#?} -> Stdio {{\n{:#?}\n}}",
                        res.io.stdin,
                        res.io.stdout
                    );
                }
            };
        }
        Ok(self)
    }

    /*
     * Mount init disk to host filesystem.
     */
    pub fn mount(&self) -> Result<&Self, VirshleError> {
        let disk_dir = self.vm.get_disk_dir()?;
        let source = format!("{disk_dir}/pipelight-init");

        let mount_dir = self.vm.get_mount_dir()?;
        let target = format!("{mount_dir}/pipelight-init");

        // Ensure mounting directory exists and nothing is already mounted.
        self.umount().ok();
        fs::create_dir_all(&target)?;

        let mut commands = vec![];

        // TODO(): add systemd unit mountcap.
        // Mount need root priviledge
        #[cfg(debug_assertions)]
        commands.push(format!(
            "sudo mount -t vfat -o loop -o gid=users -o umask=007 {source} {target}"
        ));
        #[cfg(not(debug_assertions))]
        commands.push(format!("mount -t vfat -o loop {source} {target}"));

        for cmd in commands {
            let mut proc = Process::new();
            let res = proc.stdin(&cmd).run()?;

            match res.state.status {
                Some(Status::Failed) => {
                    let message = format!("Couldn't mount init disk.");
                    let help = res.io.stderr.unwrap_or_default();
                    return Err(LibError::builder().msg(&message).help(&help).build().into());
                }
                _ => {
                    trace!(
                        "Executed {:#?} -> Stdio {{\n{:#?}\n}}",
                        res.io.stdin,
                        res.io.stdout
                    );
                }
            };
        }
        Ok(self)
    }
    /*
     * Unmount init disk from host filesystem.
     */
    pub fn umount(&self) -> Result<&Self, VirshleError> {
        let mount_dir = self.vm.get_mount_dir()?;
        let target = format!("{mount_dir}/pipelight-init");

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
                    let message = format!("Couldn't unmount init disk.");
                    let help = res.io.stderr.unwrap_or_default();
                    return Err(LibError::builder().msg(&message).help(&help).build().into());
                }
                _ => {
                    trace!(
                        "Executed {:#?} -> Stdio {{\n{:#?}\n}}",
                        res.io.stdin,
                        res.io.stdout
                    );
                }
            };
        }

        // Clean mount points
        fs::remove_dir_all(&target)?;

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
