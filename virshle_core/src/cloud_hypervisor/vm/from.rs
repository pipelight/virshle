use super::{VirshleVmConfig, Vm, VmNet};
use crate::cloud_hypervisor::{Disk, DiskTemplate};

use serde::{Deserialize, Serialize};
use std::fs;

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
use pipelight_error::{CastError, TomlError};
use virshle_error::{LibError, VirshleError};

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
    pub name: Option<String>,
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

        if let Some(name) = &e.name {
            vm.name = name.to_owned();
        }
        if let Some(uuid) = &e.uuid {
            vm.uuid = uuid.to_owned();
        }
        // Make disks
        if let Some(defs) = &e.disk {
            for def in defs {
                vm.disk.push(Disk::from(def))
            }
        } else {
            vm.disk.push(Disk {
                path: format!("{}{}{}", MANAGED_DIR.to_owned(), "/disk/", vm.uuid),
                readonly: false,
            })
        }
        vm
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
        let res = toml::from_str::<VmTemplate>(string);

        let item: VmTemplate = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlError(TomlError::new(e, string));
                return Err(err.into());
            }
        };
        let mut item = Vm::from(&item);
        item.update();
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
            [net.tap]
            name = "macvtap0"

            [[net]]
            [net.bridge]
            name = "virshlebr0"

            "#;
        let item = Vm::from_toml(&toml)?;
        println!("{:#?}", item);
        Ok(())
    }
}
