mod crud;
mod from;
mod getters;
mod interface;
mod to;

pub use from::NetTemplate;

use interface::Ip;

use super::rand::random_place;
use pipelight_exec::{Process, Status};
use serde::{Deserialize, Serialize};
use std::fs;
use tabled::{Table, Tabled};

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

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Net {
    name: String,
    // CIDR notation ip/subnet_mask
    ip: String,
    // autostart net on host boot
    enabled: bool,
    uuid: Uuid,
}

impl Default for Net {
    fn default() -> Self {
        Self {
            name: random_place().unwrap(),
            ip: "192.168.200.1/24".to_owned(),
            enabled: true,
            uuid: Uuid::new_v4(),
        }
    }
}
