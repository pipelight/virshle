use super::{Disk, NetType, Vm, VmNet};
use crate::config::MANAGED_DIR;

// Process
use pipelight_exec::{Finder, Process};

// Filesystem
use std::fs;
use std::path::Path;

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::{prelude::*, *};
use sea_orm::{
    prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult, IntoActiveModel,
};

// Http
use crate::connection::{Connection, ConnectionHandle, UnixConnection};
use crate::http_request::{Rest, RestClient};

// Ovs
use crate::network::{ip, ip::fd, ovs::OvsBridge};

// Error Handling
use log::{error, info};
use miette::{IntoDiagnostic, Result};
use virshle_error::{CastError, LibError, VirshleError};

impl Vm {
    /*
     * Create needed resources (network)
     * And start the virtual machine and .
     */
    pub async fn start(&mut self) -> Result<(), VirshleError> {
        // self.create_networks()?;

        self.start_vmm().await?;

        // Provision with user defined data
        self.add_init_disk()?;

        self.push_config_to_vmm().await?;

        let mut conn = Connection::from(self);
        let mut rest = RestClient::from(&mut conn);

        let endpoint = "/api/v1/vm.boot";
        let response = rest.put::<()>(endpoint, None).await?;

        if !response.status().is_success() {
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
        let record = database::entity::vm::ActiveModel {
            uuid: ActiveValue::Set(self.uuid.to_string()),
            name: ActiveValue::Set(self.name.clone()),
            definition: ActiveValue::Set(serde_json::to_value(&self)?),
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
        if let Some(nets) = &self.net {
            for net in nets {
                // This results in "machin_name-network_name".
                let port_name = format!("vm-{}-{}", self.name, net.name);

                match &net._type {
                    NetType::Vhost(v) => {
                        // Delete existing socket if any
                        // because ch will create one on process start.
                        let socket_path = self.get_net_socket(net)?;
                        let path = Path::new(&socket_path);
                        if path.exists() {
                            fs::remove_file(&socket_path)?;
                        }
                        // Ovs
                        // Replace existing port with a fresh one.
                        // Try to delete the port and silently fail
                        if let Some(port) = OvsBridge::get_vm_switch()?.get_port(&port_name).ok() {
                            port.delete().ok();
                        }
                        OvsBridge::get_vm_switch()?.create_dpdk_port(&port_name, &socket_path)?;
                    }
                    NetType::Tap(v) => {
                        // Ovs
                        // Replace existing port and tap device with fresh ones.
                        // Try to delete the port and silently fail.
                        if let Some(port) = OvsBridge::get_vm_switch()?.get_port(&port_name).ok() {
                            port.delete().ok();
                        }
                        OvsBridge::get_vm_switch()?.create_tap_port(&port_name)?;

                        // IP
                        // ip::tap::delete(&port_name).ok();
                        // ip::tap::create(&port_name)?;

                        ip::up(&port_name)?;
                    }
                };
            }
        }
        Ok(())
    }
}
