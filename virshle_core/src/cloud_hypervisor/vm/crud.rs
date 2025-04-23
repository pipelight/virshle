use super::{Disk, Vm, VmNet};
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

// Ovs
use crate::network::Ovs;

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use virshle_error::{CastError, LibError, VirshleError};

impl Vm {
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
    pub fn create_networks(&self) -> Result<Vec<VmNet>, VirshleError> {
        let mut net: Vec<VmNet> = vec![];
        if let Some(networks) = &self.net {
            net = networks.to_owned();
            for e in networks {
                // Delete existing socket if any
                // because ch will create one on process start.
                let socket_path = self.get_net_socket(e)?;
                let path = Path::new(&socket_path);
                if path.exists() {
                    fs::remove_file(&socket_path)?;
                }
                let port_name = format!("{}-{}", self.name, e.name);
                Ovs::create_vm_port(&port_name, &socket_path)?;
            }
        }
        Ok(net)
    }
    /*
     * Remove a vm definition from database.
     * And delete vm resources and process.
     */
    pub async fn delete(&self) -> Result<Self, VirshleError> {
        // Remove process and artifacts.
        self.delete_ch_proc()?;
        // Remove vm disks
        self.delete_disks()?;
        // Remove vm networks
        self.delete_networks()?;
        // Finally Remove db networks
        self.delete_db_record().await?;

        info!("Deleted vm {}", self.name);
        Ok(self.to_owned())
    }

    /*
     * Remove vm record from database.
     */
    pub async fn delete_db_record(&self) -> Result<Self, VirshleError> {
        let db = connect_db().await?;
        let record = database::prelude::Vm::find()
            .filter(database::entity::vm::Column::Name.eq(&self.name))
            .one(&db)
            .await?;
        if let Some(record) = record {
            database::prelude::Vm::delete(record.into_active_model())
                .exec(&db)
                .await?;
        }
        Ok(self.to_owned())
    }
    /*
     * Remove vm disks file from filesystem.
     */
    pub fn delete_disks(&self) -> Result<Vec<Disk>, VirshleError> {
        for disk in &self.disk {
            let path = Path::new(&disk.path);
            if path.exists() {
                fs::remove_file(&disk.path)?;
            }
        }
        Ok(self.disk.to_owned())
    }
    /*
     * Remove network from filesystem (and ovs configuration).
     */
    pub fn delete_networks(&self) -> Result<Vec<VmNet>, VirshleError> {
        let mut net: Vec<VmNet> = vec![];
        if let Some(networks) = &self.net {
            net = networks.to_owned();
            for e in networks {
                let socket_path = self.get_net_socket(&e)?;
                let path = Path::new(&socket_path);
                if path.exists() {
                    fs::remove_file(&socket_path)?;
                }
                let port_name = format!("{}-{}", self.name, e.name);
                Ovs::delete_vm_port(&port_name)?;
            }
        }
        Ok(net)
    }
    /*
     * Remove running vm hypervisor process if any
     * and assiociated socket.
     */
    pub fn delete_ch_proc(&self) -> Result<(), VirshleError> {
        Finder::new()
            .seed("cloud-hypervisor")
            .seed(&self.uuid.to_string())
            .search_no_parents()?
            .kill()?;

        let socket = &self.get_socket()?;
        let path = Path::new(&socket);
        if path.exists() {
            fs::remove_file(&socket)?;
        }

        Ok(())
    }
    /*
     * Remove vm working directory and dependencies filetree.
     * Usually at : `/var/lib/virshle/vm/{vm_uuid}`.
     */
    pub fn delete_filetree(&self) -> Result<(), VirshleError> {
        let directory = self.get_dir()?;
        let path = Path::new(&directory);
        if path.exists() {
            fs::remove_dir_all(&directory)?;
        }
        Ok(())
    }
}
