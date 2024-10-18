use super::Net;

use serde::{Deserialize, Serialize};
use std::fs;

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::{prelude::*, *};
use sea_orm::{
    prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult, IntoActiveModel,
};

// Error Handling
use log::info;
use miette::{Error, IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct NetTemplate {
    name: Option<String>,
    // CIDR notation ip/subnet_mask
    ip: Option<String>,
    // autostart net on host boot
    enabled: Option<bool>,
}

impl From<net::Model> for Net {
    fn from(record: net::Model) -> Self {
        let definition: Net = serde_json::from_value(record.definition).unwrap();
        Self {
            uuid: Uuid::parse_str(&record.uuid).unwrap(),
            name: record.name,
            ..definition
        }
    }
}
impl From<&NetTemplate> for Net {
    fn from(e: &NetTemplate) -> Self {
        let mut net = Self {
            ..Default::default()
        };
        if let Some(name) = &e.name {
            net.name = name.to_owned();
        }
        if let Some(ip) = &e.ip {
            net.ip = ip.to_owned();
        }
        net
    }
}
impl Net {
    pub fn from_file(file_path: &str) -> Result<Self, VirshleError> {
        let string = fs::read_to_string(file_path)?;
        Self::from_toml(&string)
    }
    pub fn from_toml(string: &str) -> Result<Self, VirshleError> {
        let res = toml::from_str::<NetTemplate>(&string);
        let item: NetTemplate = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlError(TomlError::new(e, &string));
                return Err(err.into());
            }
        };
        let item = Net::from(&item);
        Ok(item)
    }
}
