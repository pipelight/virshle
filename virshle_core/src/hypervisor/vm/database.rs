use super::account::Account;
use super::Vm;
use crate::config::{Config, MANAGED_DIR};

// Init disk
use super::init::UserData;

// Time
use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::{
    // prelude::*,
    *,
};
use sea_orm::{
    prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult, IntoActiveModel,
};

use std::fs;
use std::path::Path;

// Error Handling
use miette::{IntoDiagnostic, Result};
use tracing::info;
use virshle_error::{LibError, VirshleError};

impl Vm {
    pub fn db(&self) -> VmDbMethods {
        VmDbMethods { vm: self }
    }
}
pub struct VmDbMethods<'a> {
    pub vm: &'a mut Vm,
}

impl VmDbMethods<'_> {
    /// Create vm record and persist into database.
    pub async fn create(&mut self, user_data: Option<UserData>) -> Result<Vm, VirshleError> {
        let db = connect_db().await?;

        // Vm record
        let now: NaiveDateTime = Utc::now().naive_utc();
        let vm = database::entity::vm::ActiveModel {
            uuid: ActiveValue::Set(self.vm.uuid.to_string()),
            name: ActiveValue::Set(self.vm.name.clone()),
            definition: ActiveValue::Set(serde_json::to_value(&self.vm)?),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
            ..Default::default()
        };
        let vm_insert_result: InsertResult<vm::ActiveModel> =
            database::prelude::Vm::insert(vm.clone()).exec(&db).await?;
        self.vm.id = Some(vm_insert_result.last_insert_id as u64);

        // Retrieve account and link Vm to account.
        match user_data {
            None => {}
            Some(user_data) => {
                if let Some(mut account) = user_data.account {
                    // Retrieve account.
                    Account::get_or_create(&mut account).await?;
                    // Create a record on junction table.
                    let junction_record = database::entity::account_vm::ActiveModel {
                        account_id: ActiveValue::Set(account.id.unwrap()),
                        vm_id: ActiveValue::Set(self.vm.id.unwrap() as i32),
                    };
                    database::prelude::AccountVm::insert(junction_record)
                        .exec(&db)
                        .await?;
                }
            }
        };

        Ok(self.vm.to_owned())
    }
    /// Remove Vm record from database.
    pub async fn delete(&self) -> Result<Vm, VirshleError> {
        let db = connect_db().await?;
        let vm_record = database::prelude::Vm::find()
            .filter(database::entity::vm::Column::Name.eq(&self.vm.name))
            .one(&db)
            .await?;

        if let Some(vm_record) = &vm_record {
            // Delete AccountVm junction record(s).
            database::prelude::AccountVm::delete_many()
                .filter(account_vm::Column::VmId.eq(vm_record.id))
                .exec(&db)
                .await?;
            // Delete Vm record
            database::prelude::Vm::delete(vm_record.clone().into_active_model())
                .exec(&db)
                .await?;
        }
        Ok(self.vm.to_owned())
    }
}
