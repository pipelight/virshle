use serde::{Deserialize, Serialize};
use uuid::Uuid;

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::{prelude, *};
use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::{
    prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult, IntoActiveModel,
};

// Error handling
use log::{debug, info};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError};
#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Account {
    pub id: Option<i32>,
    pub uuid: Uuid,
    pub user: Option<AccountUser>,
}
#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct AccountUser {
    pub name: String,
    pub public_key: String,
}

impl Account {
    /*
     * Create account record and persist into database.
     */
    async fn create_db_record(account: &mut Account) -> Result<Self, VirshleError> {
        let record = database::entity::account::ActiveModel {
            uuid: ActiveValue::Set(account.uuid.to_string()),
            ..Default::default()
        };

        let db = connect_db().await?;

        let res: InsertResult<account::ActiveModel> =
            database::prelude::Account::insert(record).exec(&db).await?;
        account.id = Some(res.last_insert_id);

        Ok(account.to_owned())
    }
    /*
     * Remove vm record from database.
     */
    pub async fn delete_db_record(&self) -> Result<Self, VirshleError> {
        let db = connect_db().await?;
        let record = database::prelude::Account::find()
            .filter(database::entity::account::Column::Uuid.eq(self.uuid))
            .one(&db)
            .await?;
        if let Some(record) = record {
            database::prelude::Account::delete(record.into_active_model())
                .exec(&db)
                .await?;
        }
        Ok(self.to_owned())
    }
    pub async fn get_or_create(&mut self) -> Result<Self, VirshleError> {
        info!("[start] retrieve existing account");
        match Self::get_by_uuid(&self.uuid).await {
            Ok(v) => {
                info!("[end] found existing account");
                *self = v;
                Ok(self.to_owned())
            }
            Err(e) => Self::create_db_record(self).await,
        }
    }
    pub async fn get_by_uuid(uuid: &Uuid) -> Result<Self, VirshleError> {
        // Retrieve from database
        let db = connect_db().await.unwrap();
        let record = database::prelude::Account::find()
            .filter(database::entity::account::Column::Uuid.eq(uuid.to_string()))
            .one(&db)
            .await?;

        match record {
            Some(record) => Ok(Self {
                id: Some(record.id),
                uuid: Uuid::parse_str(&record.uuid)?,
                ..Default::default()
            }),
            None => {
                let message = format!("Couldn't find an account with uuid: {}", uuid);
                let help = "Are you sure this account exist?";
                return Err(LibError::builder().msg(&message).help(help).build().into());
            }
        }
    }
}
