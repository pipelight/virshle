use super::{DiskConfig, VirshleVmConfig, Vm};

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
        let config: Vm = serde_json::from_value(record.config).unwrap();
        Self {
            uuid: Uuid::parse_str(&record.uuid).unwrap(),
            name: record.name,
            ..config
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
    pub uuid: Uuid,
    pub disk: Option<Vec<DiskConfig>>,
    pub config: Option<VirshleVmConfig>,
}

impl Vm {
    /*
     * Create a vm from a file containing a Toml definition.
     */
    pub fn from_file(file_path: &str) -> Result<Self, VirshleError> {
        let string = fs::read_to_string(file_path)?;
        let res = toml::from_str::<Self>(&string);
        let mut item = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlError(TomlError::new(e, &string));
                return Err(err.into());
            }
        };
        item.update();
        Ok(item)
    }

    /*
     * Create a vm from a file containing a Toml definition.
     */
    pub fn from_template(file_path: &str) -> Result<Self, VirshleError> {
        let string = fs::read_to_string(file_path)?;
        let res = toml::from_str::<VmTemplate>(&string);

        let mut item = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlError(TomlError::new(e, &string));
                return Err(err.into());
            }
        };
        // item.update();
        Ok(item)
    }
}
