use serde::{Deserialize, Serialize};

use bon::bon;
use uuid::Uuid;

// Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::*;
use sea_orm::{prelude::*, ActiveValue, InsertResult, IntoActiveModel};

// Error handling
use miette::Result;
use virshle_error::{LibError, VirshleError};

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct UserData {
    pub user: Vec<User>,
    /// Only purpose is to remain in database for further VM identification.
    pub account: Option<Account>,
}
#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct User {
    pub name: String,
    pub ssh: Option<SshParams>,
}
#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct SshParams {
    pub authorized_keys: Vec<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Account {
    pub id: Option<i32>,
    pub uuid: Uuid,
}

impl Account {
    pub async fn db(&mut self) -> Result<AccountDbMethods<'_>, VirshleError> {
        let db = connect_db().await?;
        Ok(AccountDbMethods { account: self, db })
    }
    pub async fn database() -> Result<AccountDbFunctions, VirshleError> {
        let db = connect_db().await?;
        Ok(AccountDbFunctions { db })
    }
}

pub struct AccountDbFunctions {
    db: DatabaseConnection,
}
#[bon]
impl AccountDbFunctions {
    // Retrieve from database
    #[builder(finish_fn = get)]
    pub async fn one(
        &self,
        uuid: Option<Uuid>,
        ssh_public_key: Option<String>,
    ) -> Result<Account, VirshleError> {
        let mut record = None;
        let mut params: Option<(String, String)> = None;
        match uuid {
            Some(uuid) => {
                params = Some(("uuid".to_owned(), uuid.to_string()));
                record = database::prelude::Account::find()
                    .filter(database::entity::account::Column::Uuid.eq(uuid.to_string()))
                    .one(&self.db)
                    .await?;
            }
            None => {}
        };

        if let Some(record) = record {
            let account = Account {
                id: Some(record.id),
                uuid: Uuid::parse_str(&record.uuid)?,
                ..Default::default()
            };
            return Ok(account);
        } else {
            let message = match params {
                Some((k, v)) => &format!("Couldn't find account with {k}: {v}."),
                None => "Couldn't find account.",
            };
            let help = "Are you sure this account exist?";
            return Err(LibError::builder().msg(&message).help(help).build().into());
        }
    }
}

pub struct AccountDbMethods<'a> {
    pub account: &'a mut Account,
    db: DatabaseConnection,
}
impl AccountDbMethods<'_> {
    #[tracing::instrument(skip_all, ret)]
    pub async fn get_or_create(&mut self) -> Result<Account, VirshleError> {
        let req = Account::database()
            .await?
            .one()
            .uuid(self.account.uuid)
            .get()
            .await;
        match req {
            Ok(v) => {
                *self.account = v;
                Ok(self.account.to_owned())
            }
            Err(e) => self.create().await,
        }
    }
    /// Create account record and persist into database.
    async fn create(&mut self) -> Result<Account, VirshleError> {
        let record = database::entity::account::ActiveModel {
            uuid: ActiveValue::Set(self.account.uuid.to_string()),
            ..Default::default()
        };

        let db = connect_db().await?;

        let res: InsertResult<account::ActiveModel> =
            database::prelude::Account::insert(record).exec(&db).await?;
        self.account.id = Some(res.last_insert_id);

        Ok(self.account.to_owned())
    }
    /// Remove account record from database.
    pub async fn delete(&self) -> Result<Account, VirshleError> {
        let record = database::prelude::Account::find()
            .filter(database::entity::account::Column::Uuid.eq(self.account.uuid))
            .one(&self.db)
            .await?;
        if let Some(record) = record {
            database::prelude::Account::delete(record.into_active_model())
                .exec(&self.db)
                .await?;
        }
        Ok(self.account.to_owned())
    }
}
