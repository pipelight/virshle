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
            vms.push(Self::get_by_uuid(&Uuid::parse_str(&e.uuid)?).await?)
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
            let definition_path = MANAGED_DIR.to_owned() + "/vm/" + &record.uuid.to_string();
            Self::from_file(&definition_path)
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
            .filter(database::entity::vm::Column::Uuid.eq(uuid.to_owned()))
            .one(&db)
            .await?;

        if let Some(record) = record {
            let definition_path = MANAGED_DIR.to_owned() + "/vm/" + &record.uuid.to_string();
            Self::from_file(&definition_path)
        } else {
            let message = format!("Could not find a vm with the uuid: {}", uuid);
            return Err(LibError::new(&message, "").into());
        }
    }
}
