use super::{Disk, Vm};
// Init disk
use super::UserData;

// Globals
use crate::config::init::MANAGED_DIR;

// Filesystem
use bon::bon;
use std::fs;
use std::path::Path;

// Error Handling
use miette::Result;
use tracing::{error, info, trace};
use virshle_error::VirshleError;

#[bon]
impl Vm {
    /// Add vm config to database.
    /// Resources are not created there but rather on vm start.
    #[tracing::instrument(skip_all)]
    pub async fn create(&mut self, user_data: Option<UserData>) -> Result<Self, VirshleError> {
        // Persist vm config into database
        self.db().await?.create(user_data.clone()).await?;

        // Create initial resources
        self.create_init_resources()
            .maybe_user_data(user_data)
            .exec()?;

        info!("created vm {:#?}", self.name);
        Ok(self.to_owned())
    }

    /// Start Vm
    #[builder(finish_fn = exec)]
    #[tracing::instrument(skip_all)]
    pub async fn start(&mut self, user_data: Option<UserData>) -> Result<Vm, VirshleError> {
        // Create initial resources
        self.create_init_resources()
            .maybe_user_data(user_data)
            .exec()?;

        // Start the ch process
        self.vmm().start().exec().await?;

        // Call to the ch process.
        // Provision the process with VM configuration.
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

    /// Create init disk and network before vm is booted.
    #[builder(finish_fn = exec)]
    #[tracing::instrument(skip_all)]
    pub fn create_init_resources(
        &mut self,
        user_data: Option<UserData>,
    ) -> Result<Vm, VirshleError> {
        // Create ressources
        self.add_init_disk(user_data)?;
        self.networks().create_all()?;

        Ok(self.to_owned())
    }

    /// Create init disk and network before vm is booted.
    #[tracing::instrument(skip_all)]
    pub async fn provision_ch_process(&mut self) -> Result<Vm, VirshleError> {
        // Call to the ch process.
        // Provision the process with VM configuration.
        self.vmm().api()?.create().await?;
        self.vmm().api()?.boot().await?;

        self.set_vsock_permissions().await?;
        Ok(self.to_owned())
    }

    /// Replace vm disks with a fresh disk.
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub fn replace_disk(&self, name: String) -> Result<(), VirshleError> {
        let disks: Vec<Disk> = self.disk.clone().into_iter().filter(|e| e.name == name).collect();
        if let Some(disk) = disks.first() {
            let path = Path::new(&disk.path);
            if path.exists() {
                // remove old disk
                fs::remove_file(&disk.path)?;

                // create fresh disk
                let filename = Path::new(&disk.path)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                let source = format!("{MANAGED_DIR}/cache/{}",filename);
                fs::copy(&source, &disk.path)?;
            }
        }
        Ok(())
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
