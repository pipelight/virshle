use super::account::Account;
use super::{Disk, NetType, Vm, VmNet};
use crate::ovs::OvsPort;

// Process
use pipelight_exec::{Finder, Process};

// Filesystem
use std::fs;
use std::path::Path;

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::{prelude, *};
use chrono::{DateTime, NaiveDateTime, Utc};
use sea_orm::{
    prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult, IntoActiveModel,
};

// Http
use crate::connection::{Connection, ConnectionHandle, UnixConnection};
use crate::http_request::{Rest, RestClient};

// Ovs
use crate::network::{ip, ip::fd, ovs::OvsBridge, utils};

// Init disk
use super::UserData;

// Error Handling
use log::{error, info, trace};
use miette::{IntoDiagnostic, Result};
use virshle_error::{CastError, LibError, VirshleError};

impl Vm {
    /// Add vm config to database.
    /// Resources are not created there but rather on vm start.
    pub async fn create(&mut self, user_data: Option<UserData>) -> Result<Self, VirshleError> {
        // Persist vm config into database
        self.create_db_record(user_data).await?;
        Ok(self.to_owned())
    }

    /// Create vm record and persist into database.
    async fn create_db_record(
        &mut self,
        user_data: Option<UserData>,
    ) -> Result<Self, VirshleError> {
        let db = connect_db().await?;

        if let Some(user_data) = user_data {
            if let Some(mut account) = user_data.account {
                // Account
                Account::get_or_create(&mut account).await?;
                // Vm record
                let now: NaiveDateTime = Utc::now().naive_utc();
                let vm = database::entity::vm::ActiveModel {
                    uuid: ActiveValue::Set(self.uuid.to_string()),
                    name: ActiveValue::Set(self.name.clone()),
                    definition: ActiveValue::Set(serde_json::to_value(&self)?),
                    created_at: ActiveValue::Set(now),
                    updated_at: ActiveValue::Set(now),
                    ..Default::default()
                };

                let vm_insert_result: InsertResult<vm::ActiveModel> =
                    database::prelude::Vm::insert(vm.clone()).exec(&db).await?;
                // TODO: handle insertion error when duplicate name or uuid.
                self.id = Some(vm_insert_result.last_insert_id as u64);

                // Junction table record
                let junction_record = database::entity::account_vm::ActiveModel {
                    account_id: ActiveValue::Set(account.id.unwrap()),
                    vm_id: ActiveValue::Set(self.id.unwrap() as i32),
                };
                database::prelude::AccountVm::insert(junction_record)
                    .exec(&db)
                    .await?;
            }
        } else {
            // Vm record
            let now: NaiveDateTime = Utc::now().naive_utc();
            let vm = database::entity::vm::ActiveModel {
                uuid: ActiveValue::Set(self.uuid.to_string()),
                name: ActiveValue::Set(self.name.clone()),
                definition: ActiveValue::Set(serde_json::to_value(&self)?),
                created_at: ActiveValue::Set(now),
                updated_at: ActiveValue::Set(now),
                ..Default::default()
            };

            let vm_insert_result: InsertResult<vm::ActiveModel> =
                database::prelude::Vm::insert(vm.clone()).exec(&db).await?;
            self.id = Some(vm_insert_result.last_insert_id as u64);
        }

        Ok(self.to_owned())
    }

    /*
     * Add network ports to ovs config.
     */
    pub fn create_networks(&self) -> Result<(), VirshleError> {
        trace!("creating networks for vm {:#?}", self.name);

        // Remove old networks
        self.delete_networks()?;

        if let Some(nets) = &self.net {
            for net in nets {
                // This results in "machin_name-network_name".
                let port_name = format!("vm-{}--{}", self.name, net.name);

                match &net._type {
                    // Not working on ovs-bridge of type "system"
                    // bridge must be of type "netdev"
                    NetType::Vhost(v) => {
                        let socket_path = self.get_net_socket(&net)?;
                        OvsBridge::get_vm_switch()?.create_dpdk_port(&port_name, &socket_path)?;
                    }
                    // Not working on ovs-bridge of type "netdev"
                    // bridge must be of type "system"
                    NetType::Tap(v) => {
                        // Create tap device
                        ip::tap::create(&port_name)?;
                        ip::up(&port_name)?;

                        // Link to ovs bridge
                        let vmbr = OvsBridge::get_vm_switch()?;
                        // Silently try to delete old port if any.
                        match OvsPort::get_by_name(&port_name) {
                            Ok(v) => {
                                v.delete()?;
                            }
                            Err(_) => {}
                        };
                        vmbr.create_tap_port(&port_name)?;
                    }
                    // Not working on ovs-bridge of type "netdev"
                    // bridge must be of type "system"
                    NetType::MacVTap(v) => {
                        // Create macvtap device
                        ip::tap::create_macvtap(&port_name)?;
                        ip::up(&port_name)?;
                    }
                };
            }
        }
        Ok(())
    }
}
