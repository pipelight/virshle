use super::Vm;

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

impl Vm {
    /*
     * Get all Vm from virshle database.
     */
    pub async fn get_all() -> Result<Vec<Vm>, VirshleError> {
        let db = connect_db().await?;
        let records: Vec<database::entity::vm::Model> =
            database::prelude::Vm::find().all(&db).await?;

        let mut vms: Vec<Vm> = vec![];
        for e in records {
            let vm: Vm = serde_json::from_value(e.definition)?;
            vms.push(vm)
        }
        Ok(vms)
    }
    /*
     * Get a Vm definition from its name.
     */
    pub async fn get_by_name(name: &str) -> Result<Self, VirshleError> {
        // Retrive from database
        let db = connect_db().await.unwrap();
        let record = database::prelude::Vm::find()
            .filter(database::entity::vm::Column::Name.eq(name))
            .one(&db)
            .await?;

        if let Some(record) = record {
            let vm: Vm = serde_json::from_value(record.definition)?;
            return Ok(vm);
        } else {
            let message = format!("Could not find a vm with the name: {}", name);
            return Err(LibError::new(&message, "").into());
        }
    }
    /*
     * Get a Vm definition from its uuid.
     */
    pub async fn get_by_uuid(uuid: &Uuid) -> Result<Self, VirshleError> {
        // Retrive from database
        let db = connect_db().await.unwrap();
        let record = database::prelude::Vm::find()
            .filter(database::entity::vm::Column::Uuid.eq(uuid.to_string()))
            .one(&db)
            .await?;

        if let Some(record) = record {
            let vm: Vm = serde_json::from_value(record.definition)?;
            return Ok(vm);
        } else {
            let message = format!("Could not find a vm with the uuid: {}", uuid);
            return Err(LibError::new(&message, "").into());
        }
    }
}
