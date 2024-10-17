use super::interface::{Ip, LinkState};
use super::Net;

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

impl Net {
    /*
     * Get all Net from virshle database.
     */
    pub async fn get_all() -> Result<Vec<Net>, VirshleError> {
        let db = connect_db().await?;
        let records: Vec<database::entity::net::Model> =
            database::prelude::Net::find().all(&db).await?;

        let mut nets: Vec<Net> = vec![];
        for e in records {
            let net: Net = serde_json::from_value(e.definition)?;
            nets.push(net)
        }
        Ok(nets)
    }
    /*
     * Get a Net definition from its name.
     */
    pub async fn get_by_name(name: &str) -> Result<Self, VirshleError> {
        // Retrive from database
        let db = connect_db().await.unwrap();
        let record = database::prelude::Net::find()
            .filter(database::entity::net::Column::Name.eq(name))
            .one(&db)
            .await?;

        if let Some(record) = record {
            let net: Net = serde_json::from_value(record.definition)?;
            return Ok(net);
        } else {
            let message = format!("Could not find a net with the name: {}", name);
            return Err(LibError::new(&message, "").into());
        }
    }
    /*
     * Get a Net definition from its uuid.
     */
    pub async fn get_by_uuid(uuid: &Uuid) -> Result<Self, VirshleError> {
        // Retrive from database
        let db = connect_db().await.unwrap();
        let record = database::prelude::Net::find()
            .filter(database::entity::net::Column::Uuid.eq(uuid.to_string()))
            .one(&db)
            .await?;

        if let Some(record) = record {
            let net: Net = serde_json::from_value(record.definition)?;
            return Ok(net);
        } else {
            let message = format!("Could not find a net with the uuid: {}", uuid);
            return Err(LibError::new(&message, "").into());
        }
    }
    /*
     * Get interface state.
     */
    pub fn get_state(&self) -> Result<LinkState, VirshleError> {
        Ip::get_interface_state(&self.name)
    }
}
