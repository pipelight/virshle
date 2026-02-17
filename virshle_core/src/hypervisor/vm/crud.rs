use super::{Disk, Vm};

// Filesystem
use std::fs;
use std::path::Path;

// Init disk
use super::UserData;

// Error Handling
use miette::Result;
use tracing::{error, info, trace};
use virshle_error::VirshleError;

impl Vm {
    /// Add vm config to database.
    /// Resources are not created there but rather on vm start.
    #[tracing::instrument(skip_all)]
    pub async fn create(&mut self, user_data: Option<UserData>) -> Result<Self, VirshleError> {
        // Persist vm config into database
        self.db().await?.create(user_data).await?;

        info!("created vm {:#?}", self.name);
        Ok(self.to_owned())
    }
    /// Start Vm
    #[tracing::instrument(skip_all)]
    pub async fn start(
        &mut self,
        user_data: Option<UserData>,
        attach: Option<bool>,
    ) -> Result<Vm, VirshleError> {
        // Create ressources
        self.add_init_disk(user_data)?;
        self.networks().create_all()?;

        self.vmm().start(attach).await?;
        self.vmm().api()?.create().await?;
        self.vmm().api()?.boot().await?;

        self.set_vsock_permissions().await?;

        info!("started vm {:#?}", self.name);
        Ok(self.to_owned())
    }
    /// Remove a vm definition from database.
    /// And delete vm resources and process.
    #[tracing::instrument(skip_all)]
    pub async fn delete(&mut self) -> Result<Self, VirshleError> {
        // Remove process and artifacts.
        self.vmm().kill_process()?;
        // Remove vm networks
        self.networks().delete_all()?;
        // Soft lease deletion
        self.networks().leases().delete_all().await.ok();
        // Remove vm disks
        self.delete_disks()?;
        // Delete vm directory tree
        self.delete_filetree()?;
        // Finally Remove db record
        self.db().await?.delete().await?;

        info!("deleted vm {}", self.name);
        Ok(self.to_owned())
    }

    /// Shut the virtual machine down and removes artifacts.
    /// Should silently fail when vm is already down.
    #[tracing::instrument(skip_all)]
    pub async fn shutdown(&self) -> Result<Self, VirshleError> {
        self.vmm().api()?.shutdown().await?;
        // Remove ch process
        self.vmm().kill_process()?;
        // Remove network ports
        self.networks().delete_all()?;

        info!("stopped vm {}", self.name);
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

    /// Remove Vm working directory and dependencies filetree.
    /// Usually at : `/var/lib/virshle/vm/{vm_uuid}`.
    pub fn delete_filetree(&self) -> Result<(), VirshleError> {
        let directory = self.get_dir()?;
        let path = Path::new(&directory);
        if path.exists() {
            fs::remove_dir_all(&directory)?;
        }
        Ok(())
    }
}
