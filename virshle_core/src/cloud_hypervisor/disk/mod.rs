// Struct
use super::Vm;

// Templating engine
use convert_case::{Case, Casing};

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
use log::info;
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct DiskTemplate {
    pub name: String,
    pub path: String,
    pub size: Option<String>,
    pub readonly: Option<bool>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Disk {
    pub name: String,
    pub path: String,
    pub readonly: Option<bool>,
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
        let disk_dir = e.vm.get_disk_dir().unwrap();
        let path = format!("{disk_dir}/pipelight-init");
        Self {
            name: "init".to_owned(),
            path,
            ..Default::default() // readonly: Some(true),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct SshData {
    user: String,
    authorized_keys: Vec<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct UserData {
    hostname: String,
    ipv6: Option<String>,
    ssh: Option<Vec<SshData>>,
}

impl UserData {
    /*
     * Convert user-data into a pipelight configuration file.
     */
    pub fn to_pipelight_toml_config(&self) -> Result<String, VirshleError> {
        let mut p_config = r#"
        [[pipelines]]
        name = "init"
        "#
        .to_owned();

        // Add hostname
        let hostname = "vm-".to_owned() + &self.hostname;
        p_config += &format!(
            r#"
        [[pipelines.steps]]
        name = "set hostname"
        commands = [
            "sysctl -w kernel.hostname='{hostname}'"
        ]
        "#
        );

        // Add public ipv6
        if let Some(ipv6) = &self.ipv6 {
            let interface = "ens3";
            p_config += &format!(
                r#"
            [[pipelines.steps]]
            name = "set ipv6"
            commands = [
                "source ./init.env && ip a $IPV6 add dev {interface}"
            ]
            "#,
            );
        }

        // Add ssh authorized_keys
        if let Some(ssh) = &self.ssh {
            for data in ssh {
                let username = data.user.to_case(Case::Upper);

                let keys = data.authorized_keys.to_owned();
                let mut commands = vec![];
                for key in keys {
                    commands.push(format!(
                        r#"
                        "cat key >> /etc/ssh/authorized_keys.d/{username}"
                    "#
                    ));
                }
                let commands = commands.join(",");

                p_config += &format!(
                    r#"
                [[pipelines.steps]]
                name = "set ipv6"
                commands = [
                    "touch /etc/ssh/authorized_keys.d/{username}",
                    {commands}
                ]
                "#
                );
            }
        }
        Ok(p_config)
    }
}

impl InitDisk<'_> {
    /*
     * Write pipelight configuration file to init disk.
     */
    pub fn write_init_files(&self) -> Result<&Self, VirshleError> {
        let user_data = UserData {
            hostname: self.vm.name.to_owned(),
            ..Default::default()
        };

        let mount_dir = self.vm.get_mount_dir()?;
        let target = format!("{mount_dir}/pipelight-init/pipelight.toml");

        // Remove old pipeline
        let path = Path::new(&target);
        if path.exists() {
            fs::remove_file(path)?;
        }
        // Write file to disk
        let p_config = user_data.to_pipelight_toml_config()?;
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
                    info!("{:#?}", res.io.stdout);
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
                    info!("{:#?}", res.io.stdout);
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
                    info!("{:#?}", res.io.stdout);
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
        init_disk.create()?.mount()?.write_init_files()?.umount()?;

        // println!("{:#?}", res);
        Ok(())
    }

    #[test]
    fn test_pipelight_config_render() -> Result<()> {
        let user_data = UserData {
            hostname: "vm-nixos".to_owned(),
            ..Default::default()
        };
        let res = user_data.to_pipelight_toml_config()?;

        println!("{}", res);
        Ok(())
    }
}
