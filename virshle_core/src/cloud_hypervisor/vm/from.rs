use super::{VirshleVmConfig, Vm, VmNet};
use crate::cloud_hypervisor::{Disk, DiskTemplate};

use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::{prelude::*, *};
use sea_orm::{prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult};

// Global configuration
use crate::config::MANAGED_DIR;

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use virshle_error::{CastError, LibError, TomlError, VirshleError, WrapError};

impl From<vm::Model> for Vm {
    fn from(record: vm::Model) -> Self {
        let definition: Vm = serde_json::from_value(record.definition).unwrap();
        Self {
            uuid: Uuid::parse_str(&record.uuid).unwrap(),
            name: record.name,
            ..definition
        }
    }
}

/*
* A partial Vm definition, with optional disk, network...
* All those usually mandatory fields will be handled by virshle with
* autoconfigured default.
*/
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct VmTemplate {
    pub name: String,
    pub vcpu: u64,
    pub vram: u64,
    pub uuid: Option<Uuid>,
    pub disk: Option<Vec<DiskTemplate>>,
    pub net: Option<Vec<VmNet>>,
    pub config: Option<VirshleVmConfig>,
}
impl From<&VmTemplate> for Vm {
    fn from(e: &VmTemplate) -> Self {
        let mut vm = Vm {
            vcpu: e.vcpu,
            vram: e.vram,
            net: e.net.clone(),
            ..Default::default()
        };
        create_resources(e, &mut vm).unwrap();
        vm
    }
}
pub fn create_resources(template: &VmTemplate, vm: &mut Vm) -> Result<(), VirshleError> {
    if let Some(disks) = &template.disk {
        for disk in disks {
            let source = shellexpand::tilde(&disk.path).to_string();
            let target = format!(
                "{}{}{}_{}.img",
                MANAGED_DIR.to_owned(),
                "/disk/",
                vm.uuid,
                disk.name
            );

            // Create disk on host drive
            let file = fs::File::create(&target)?;
            fs::copy(&source, &target)?;

            // Set permissions
            let metadata = file.metadata()?;
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&target, perms)?;

            // Push disk path to vm def
            vm.disk.push(Disk {
                name: disk.name.clone(),
                path: target,
                readonly: Some(false),
            })
        }
    }
    Ok(())
}
impl VmTemplate {
    pub fn from_file(file_path: &str) -> Result<Self, VirshleError> {
        let string = fs::read_to_string(file_path)?;
        Self::from_toml(&string)
    }
    pub fn from_toml(string: &str) -> Result<Self, VirshleError> {
        let res = toml::from_str::<Self>(string);
        let item: Self = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlError(TomlError::new(e, string));
                return Err(err.into());
            }
        };
        Ok(item)
    }
}
impl Vm {
    /*
     * Create a vm from a file containing a Toml definition.
     */
    pub fn from_file(file_path: &str) -> Result<Self, VirshleError> {
        let string = fs::read_to_string(file_path)?;
        Self::from_toml(&string)
    }
    pub fn from_toml(string: &str) -> Result<Self, VirshleError> {
        let res = toml::from_str::<Self>(string);
        let item: Self = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlError(TomlError::new(e, string));
                let err = WrapError::builder()
                    .msg("Couldn't convert toml string to a valid vm")
                    .help("")
                    .origin(err.into())
                    .build();
                return Err(err.into());
            }
        };
        Ok(item)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn make_vm_from_template() -> Result<()> {
        let toml = "
            vcpu = 1
            vram = 2

            [config]
            autostart = true
        ";

        let item = Vm::from_toml(&toml)?;
        println!("{:#?}", item);
        Ok(())
    }
    #[test]
    fn make_vm_from_definition_with_ids() -> Result<()> {
        let toml = r#"
            name = "default_xs"
            uuid = "b30458d1-7c7f-4d06-acc2-159e43892e87"

            vcpu = 1
            vram = 2

            [[net]]
            [net.vhost]

            "#;
        let item = Vm::from_toml(&toml)?;
        println!("{:#?}", item);
        Ok(())
    }
}
