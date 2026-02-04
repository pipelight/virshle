use super::{Disk, NetType, Vm, VmNet};
use crate::config::{VirshleConfig, MANAGED_DIR};
use crate::network::dhcp::{DhcpType, FakeDhcp, KeaDhcp};

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
use crate::network::{ip, ip::fd, ovs::OvsBridge};

// Error Handling
use miette::{IntoDiagnostic, Result};
use tracing::info;
use virshle_error::{CastError, LibError, VirshleError};

impl Vm {
    /// Remove a vm definition from database.
    /// And delete vm resources and process.
    #[tracing::instrument(skip_all)]
    pub async fn delete(&self) -> Result<Self, VirshleError> {
        // Remove process and artifacts.
        self.vmm().kill_process()?;
        // Remove vm networks
        self.delete_networks()?;
        // Soft lease deletion
        self.delete_leases().await.ok();
        // Remove vm disks
        self.delete_disks()?;
        // Delete vm directory tree
        self.delete_filetree()?;
        // Finally Remove db record
        self.delete_db_record().await?;

        info!("deleted vm {}", self.name);
        Ok(self.to_owned())
    }

    /// Remove vm record from database.
    pub async fn delete_db_record(&self) -> Result<Self, VirshleError> {
        let db = connect_db().await?;
        let vm_record = database::prelude::Vm::find()
            .filter(database::entity::vm::Column::Name.eq(&self.name))
            .one(&db)
            .await?;

        if let Some(vm_record) = &vm_record {
            // Delete AccountVm junction records.
            database::prelude::AccountVm::delete_many()
                .filter(account_vm::Column::VmId.eq(vm_record.id))
                .exec(&db)
                .await?;

            // Delete assiociated leases.
            match VirshleConfig::get()?.dhcp {
                Some(DhcpType::Fake(fake_dhcp)) => {
                    FakeDhcp::delete_leases(vm_record.id).await?;
                }
                Some(DhcpType::Kea(kea_dhcp)) => {
                    kea_dhcp.delete_leases(&self.name).await?;
                }
                _ => {}
            }
            // Delete Vm
            database::prelude::Vm::delete(vm_record.clone().into_active_model())
                .exec(&db)
                .await?;
        }
        Ok(self.to_owned())
    }

    /// Remove vm disks file from filesystem.
    pub fn delete_disks(&self) -> Result<Vec<Disk>, VirshleError> {
        for disk in &self.disk {
            let path = Path::new(&disk.path);
            if path.exists() {
                fs::remove_file(&disk.path)?;
            }
        }
        Ok(self.disk.to_owned())
    }

    /// Remove network from filesystem (and ovs configuration).
    pub fn delete_networks(&self) -> Result<(), VirshleError> {
        if let Some(networks) = &self.net {
            for net in networks {
                // This results in "machin_name-network_name".
                let port_name = format!("vm-{}--{}", self.name, net.name);

                // Ovs
                // Replace existing port with a fresh one.
                // Try to delete the port and silently fail
                if let Some(port) = OvsBridge::get_vm_switch()?.get_port(&port_name).ok() {
                    port.delete().ok();
                }

                match &net._type {
                    NetType::Vhost(_) => {
                        // Delete existing socket if any
                        // because ch will create one on process start.
                        let socket_path = self.get_net_socket(&net)?;
                        let path = Path::new(&socket_path);
                        if path.exists() {
                            fs::remove_file(&socket_path)?;
                        }
                    }
                    NetType::Tap(_) | NetType::MacVTap(_) => {
                        // IP
                        ip::tap::delete(&port_name).ok();
                    }
                };
            }
        }
        Ok(())
    }
    /// Remove vm working directory and dependencies filetree.
    /// Usually at : `/var/lib/virshle/vm/{vm_uuid}`.
    pub fn delete_filetree(&self) -> Result<(), VirshleError> {
        let directory = self.get_dir()?;
        let path = Path::new(&directory);
        if path.exists() {
            fs::remove_dir_all(&directory)?;
        }
        Ok(())
    }

    /// Delete vm associated dhcp leases.
    pub async fn delete_leases(&self) -> Result<(), VirshleError> {
        match VirshleConfig::get()?.dhcp {
            Some(DhcpType::Kea(kea_dhcp)) => {
                kea_dhcp.delete_ipv6_leases_by_name(&self.name).await?;
            }
            _ => {}
        };
        Ok(())
    }
}
