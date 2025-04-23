use super::{Vm, VmConfigPlus, VmNet};
use crate::cloud_hypervisor::{Disk, DiskTemplate};

// Pretty print
use bat::PrettyPrinter;
use crossterm::{execute, style::Stylize, terminal::size};
use log::{info, log_enabled, Level};

use serde::{Deserialize, Serialize};

// Filesystem manipulation
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::{prelude::*, *};
use sea_orm::{prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult};

// Global configuration
use crate::config::MANAGED_DIR;

// Error Handling
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
    pub config: Option<VmConfigPlus>,
}
impl From<&VmTemplate> for Vm {
    fn from(e: &VmTemplate) -> Self {
        let mut vm = Vm {
            vcpu: e.vcpu,
            vram: e.vram,
            net: e.net.clone(),
            ..Default::default()
        };
        ensure_directories(e, &mut vm).unwrap();
        create_disks(e, &mut vm).unwrap();
        vm
    }
}

/*
* Ensure vm storage directories exists on host.
*/
pub fn ensure_directories(template: &VmTemplate, vm: &mut Vm) -> Result<(), VirshleError> {
    let directories = [
        format!("{MANAGED_DIR}/vm/{}", vm.uuid),
        format!("{MANAGED_DIR}/vm/{}/disk", vm.uuid),
        format!("{MANAGED_DIR}/vm/{}/net", vm.uuid),
    ];
    for directory in directories {
        let path = Path::new(&directory);
        if !path.exists() {
            fs::create_dir_all(&directory)?;
        }
    }
    Ok(())
}

/*
* Copy template disks (if some)
* to vm storage directory and set file permissions.
*/
pub fn create_disks(template: &VmTemplate, vm: &mut Vm) -> Result<(), VirshleError> {
    if let Some(disks) = &template.disk {
        for disk in disks {
            let source = shellexpand::tilde(&disk.path).to_string();
            let target = format!("{MANAGED_DIR}/vm/{}/disk/{}", vm.uuid, disk.name);

            // Create disk on host drive
            let file = fs::File::create(&target)?;
            fs::copy(&source, &target)?;

            // Set permissions
            let metadata = file.metadata()?;
            let mut perms = metadata.permissions();
            perms.set_mode(0o766);
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
    pub fn to_toml(&self) -> Result<String, VirshleError> {
        let string: String = toml::to_string(self).map_err(CastError::from)?;
        if log_enabled!(Level::Warn) {
            let (cols, _) = size()?;
            let divider = "-".repeat((cols / 3).into());
            println!("{}", format!("{divider}toml{divider}").green());
            PrettyPrinter::new()
                .input_from_bytes(string.as_bytes())
                .language("toml")
                .print()?;
            println!("{}", format!("{divider}----{divider}").green());
            println!("");
        }
        Ok(string)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn display_vm_to_toml() -> Result<()> {
        let vm = Vm::default();
        let string = vm.to_toml()?;
        println!("");
        PrettyPrinter::new()
            .input_from_bytes(string.as_bytes())
            .language("toml")
            .print()
            .into_diagnostic()?;
        Ok(())
    }
    #[test]
    fn make_vm_template_from_toml() -> Result<()> {
        let toml = r#"
            name = "my_template"

            vcpu = 1
            vram = 2

            [[disk]]
            name = "os"
            path = "~/tmp/disk/template.iso"

            [[net]]
            name = "main"
            [net.type.vhost]
        "#;
        let item = VmTemplate::from_toml(&toml)?;
        println!("{:#?}", item);
        Ok(())
    }
    #[test]
    fn make_vm_from_toml() -> Result<()> {
        let toml = r#"
            name = "my_test_vm"
            uuid = "b30458d1-7c7f-4d06-acc2-159e43892e87"

            vcpu = 1
            vram = 2

            [[disk]]
            name = "os"
            path = "~/tmp/disk/uuid.iso"

            [[net]]
            name = "main"
            [net.type.vhost]
            "#;
        let item = Vm::from_toml(&toml)?;
        println!("{:#?}", item);
        Ok(())
    }
}
