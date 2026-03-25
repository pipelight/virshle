use super::Vm;

use crate::config::UserData;
use crate::hypervisor::vmm::VmState;

use bon::bon;

// Time
use chrono::{NaiveDateTime, Utc};

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::*;
use sea_orm::{prelude::*, query::*, ActiveValue, InsertResult, IntoActiveModel};

// Error Handling
use miette::Result;
use tracing::{error, info};
use virshle_error::{LibError, VirshleError, WrapError};

impl Vm {
    pub async fn db(&mut self) -> Result<VmDbMethods, VirshleError> {
        let db = connect_db().await?;
        Ok(VmDbMethods { vm: self, db })
    }
    pub async fn database() -> Result<VmDbFunctions, VirshleError> {
        let db = connect_db().await?;
        Ok(VmDbFunctions { db })
    }
}

pub struct VmDbFunctions {
    db: DatabaseConnection,
}
#[bon]
impl VmDbFunctions {
    #[builder(
        finish_fn = get, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn one(
        &self,
        id: Option<u64>,
        name: Option<String>,
        uuid: Option<Uuid>,
    ) -> Result<Vm, VirshleError> {
        let mut record = None;
        let mut params: Option<(String, String)> = None;
        match id {
            Some(id) => {
                params = Some(("id".to_owned(), id.to_string()));
                record = database::prelude::Vm::find()
                    .filter(database::entity::vm::Column::Id.eq(id.clone()))
                    .one(&self.db)
                    .await?;
            }
            None => {}
        };
        match name {
            Some(name) => {
                params = Some(("name".to_owned(), name.clone()));
                record = database::prelude::Vm::find()
                    .filter(database::entity::vm::Column::Name.eq(name))
                    .one(&self.db)
                    .await?;
            }
            None => {}
        };
        match uuid {
            Some(uuid) => {
                params = Some(("uuid".to_owned(), uuid.to_string()));
                record = database::prelude::Vm::find()
                    .filter(database::entity::vm::Column::Uuid.eq(uuid.to_string()))
                    .one(&self.db)
                    .await?;
            }
            None => {}
        };
        if let Some(record) = record {
            let vm: Vm = record.try_into()?;
            return Ok(vm);
        } else {
            let message = match params {
                Some((k, v)) => &format!("Couldn't find a vm with {}: {:#?}.", k, v),
                None => "Couldn't find a vm.",
            };
            let help = "Are you sure this vm exist?";
            return Err(LibError::builder().msg(message).help(help).build().into());
        }
    }
    // Return VMs associated with a specific account on node.
    #[builder(finish_fn = get)]
    pub async fn many(
        &self,
        vm_state: Option<VmState>,
        account_uuid: Option<Uuid>,
    ) -> Result<Vec<Vm>, VirshleError> {
        let account = match account_uuid {
            Some(account_uuid) => {
                let account: Option<database::entity::account::Model> =
                    database::prelude::Account::find()
                        .filter(
                            database::entity::account::Column::Uuid.eq(account_uuid.to_string()),
                        )
                        .one(&self.db)
                        .await?;
                account
            }
            None => None,
        };

        let records: Vec<database::entity::vm::Model> = match account {
            Some(account) => {
                account
                    .find_related(database::entity::prelude::Vm)
                    .order_by_asc(database::entity::vm::Column::CreatedAt)
                    .all(&self.db)
                    .await?
            }
            None => {
                database::prelude::Vm::find()
                    .order_by_asc(database::entity::vm::Column::CreatedAt)
                    .all(&self.db)
                    .await?
            }
        };

        let mut vms: Vec<Vm> = vec![];
        for record in records {
            let vm: Vm = record.try_into()?;
            vms.push(vm);
        }

        // Filter by state
        if let Some(vm_state) = vm_state {
            vms = Self::filter_by_state(vms, &vm_state).await?;
        }

        Ok(vms)
    }
    async fn filter_by_state(vms: Vec<Vm>, state: &VmState) -> Result<Vec<Vm>, VirshleError> {
        let mut vm_by_state: Vec<Vm> = vec![];
        for vm in vms {
            if vm.vmm().api()?.state().await? == *state {
                vm_by_state.push(vm);
            }
        }
        Ok(vm_by_state)
    }
}

pub struct VmDbMethods<'a> {
    pub vm: &'a mut Vm,
    db: DatabaseConnection,
}
impl VmDbMethods<'_> {
    /// Create vm record and persist into database.
    pub async fn create(&mut self, user_data: Option<UserData>) -> Result<Vm, VirshleError> {
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
            database::prelude::Vm::insert(vm.clone())
                .exec(&self.db)
                .await?;
        self.vm.id = Some(vm_insert_result.last_insert_id as u64);

        // Retrieve account and link Vm to account.
        match user_data {
            None => {}
            Some(user_data) => {
                if let Some(mut account) = user_data.account {
                    // Retrieve account.
                    account.db().await?.get_or_create().await?;
                    // Create a record on junction table.
                    let junction_record = database::entity::account_vm::ActiveModel {
                        account_id: ActiveValue::Set(account.id.unwrap()),
                        vm_id: ActiveValue::Set(self.vm.id.unwrap() as i32),
                    };
                    database::prelude::AccountVm::insert(junction_record)
                        .exec(&self.db)
                        .await?;
                }
            }
        };

        Ok(self.vm.to_owned())
    }
    /// Remove Vm record from database.
    pub async fn delete(&self) -> Result<Vm, VirshleError> {
        let vm_record = database::prelude::Vm::find()
            .filter(database::entity::vm::Column::Name.eq(&self.vm.name))
            .one(&self.db)
            .await?;

        if let Some(vm_record) = &vm_record {
            // Delete AccountVm junction record(s).
            database::prelude::AccountVm::delete_many()
                .filter(account_vm::Column::VmId.eq(vm_record.id))
                .exec(&self.db)
                .await?;
            // Delete Vm record
            database::prelude::Vm::delete(vm_record.clone().into_active_model())
                .exec(&self.db)
                .await?;
        }
        Ok(self.vm.to_owned())
    }

    pub async fn get_account_uuid(&self) -> Result<Uuid, VirshleError> {
        let vm_record: Option<database::entity::vm::Model> = database::prelude::Vm::find()
            .filter(database::entity::vm::Column::Uuid.eq(self.vm.uuid.to_string()))
            .order_by_asc(database::entity::vm::Column::CreatedAt)
            .one(&self.db)
            .await?;

        if let Some(vm_record) = vm_record {
            let account = vm_record
                .find_related(prelude::Account)
                .one(&self.db)
                .await?;
            if let Some(account) = account {
                return Ok(Uuid::parse_str(&account.uuid)?);
            }
        }
        let err = LibError::builder()
            .msg("Couldn't find any associated account for this vm.")
            .help("")
            .build();
        Err(err.into())
    }
}
