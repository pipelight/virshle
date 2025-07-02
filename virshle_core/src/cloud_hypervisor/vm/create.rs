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
use crate::database::entity::{prelude::*, *};
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
    pub async fn start(
        &mut self,
        user_data: Option<UserData>,
        attach: Option<bool>,
    ) -> Result<(), VirshleError> {
        self.create_networks()?;
        self.start_vmm(attach).await?;

        // Provision with user defined data
        self.add_init_disk(user_data)?;

        self.push_config_to_vmm().await?;

        let mut conn = Connection::from(self);
        let mut rest = RestClient::from(&mut conn);

        let endpoint = "/api/v1/vm.boot";
        let response = rest.put::<()>(endpoint, None).await?;

        if response.status().is_success() {
            let msg = &response.to_string().await?;
            trace!("{}", &msg);
        } else {
            let err_msg = &response.to_string().await?;
            error!("{}", &err_msg);

            let message = "Couldn't boot vm.";
            return Err(LibError::builder()
                .msg(&message)
                .help(&err_msg)
                .build()
                .into());
        }

        Ok(())
    }
    /*
     * Add vm config to database.
     * Resources are not created there but rather on vm start.
     */
    pub async fn create(&mut self) -> Result<Self, VirshleError> {
        // Persist vm config into database
        self.create_db_record().await?;
        Ok(self.to_owned())
    }

    /*
     * Create vm record and persist into database.
     */
    async fn create_db_record(&mut self) -> Result<Self, VirshleError> {
        let now: NaiveDateTime = Utc::now().naive_utc();
        let record = database::entity::vm::ActiveModel {
            uuid: ActiveValue::Set(self.uuid.to_string()),
            name: ActiveValue::Set(self.name.clone()),
            definition: ActiveValue::Set(serde_json::to_value(&self)?),

            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),

            ..Default::default()
        };

        let db = connect_db().await?;
        let res: InsertResult<vm::ActiveModel> =
            database::prelude::Vm::insert(record).exec(&db).await?;
        self.id = Some(res.last_insert_id as u64);

        Ok(self.to_owned())
    }

    /*
     * Add network ports to ovs config.
     */
    pub fn create_networks(&self) -> Result<(), VirshleError> {
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
