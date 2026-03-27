use crate::config::{Config, DhcpType};
use crate::database;
use crate::hypervisor::Vm;
use crate::network::{dhcp::KeaDhcp, ovs};

use owo_colors::OwoColorize;
use std::fs;
use std::path::Path;

// Error Handling
use log::{debug, info};
use miette::Result;
use virshle_error::{LibError, VirshleError};

/// XDG (Cross Desktop Group) directory
pub const MANAGED_DIR: &'static str = "/var/lib/virshle";
pub const CONFIG_DIR: &'static str = "/etc/virshle";

pub struct InitMethods<'a> {
    config: &'a Config,
}
impl Config {
    pub fn init(&self) -> InitMethods<'_> {
        InitMethods { config: self }
    }
}

impl InitMethods<'_> {
    /// Ensure virshle resources:
    ///   - a clean working directory and database.
    ///   - an initial configuration.
    ///   - a dedicated network virtual switch.
    pub async fn ensure_all(&self) -> Result<(), VirshleError> {
        self.directories()
            .await?
            .database()
            .await?
            .network()
            .await?;

        Ok(())
    }
    /// Ensure virshle working directories exists.
    pub async fn directories(&self) -> Result<&Self, VirshleError> {
        // Create storage/config directories
        let directories = [
            MANAGED_DIR.to_owned(),
            MANAGED_DIR.to_owned() + "/vm",
            MANAGED_DIR.to_owned() + "/cache",
            CONFIG_DIR.to_owned(),
        ];
        for directory in directories {
            let path = Path::new(&directory);
            if !path.exists() {
                fs::create_dir_all(&directory)?;
                info!("{} created virshle filetree.", "[init]".yellow(),);
            }
        }
        self._clean_directories().await?;
        Ok(self)
    }
    /// Clean orphan vm files if vm not in database.
    pub async fn _clean_directories(&self) -> Result<(), VirshleError> {
        let vms = Vm::database().await?.many().get().await?;
        let uuids: Vec<String> = vms.iter().map(|e| e.uuid.to_string()).collect();

        let path = format!("{MANAGED_DIR}/vm");
        let path = Path::new(&path);
        for entry in path.read_dir()? {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    if !uuids.contains(&entry.file_name().to_str().unwrap().to_owned()) {
                        fs::remove_dir_all(entry.path())?;
                        debug!("cleaned virshle filetree.");
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn network(&self) -> Result<&Self, VirshleError> {
        ovs::ensure_switches().await?;
        info!(
            "{} created virshle ovs network configuration.",
            "[init]".yellow(),
        );
        self._clean_leases().await?;
        Ok(self)
    }
    /// Clean dhcp leases
    pub async fn _clean_leases(&self) -> Result<&Self, VirshleError> {
        match self.config.dhcp.clone() {
            Some(DhcpType::Kea(kea_config)) => {
                let mut cli = KeaDhcp::builder().config(&kea_config).build().await?;
                cli.lease().clean().inet4(true).inet6(true).exec().await?;
                info!("{} delete unused leases", "[kea-dhcp]".yellow(),);
            }
            _ => {}
        };
        Ok(self)
    }

    pub async fn database(&self) -> Result<&Self, VirshleError> {
        database::connect_or_fresh_db().await?;
        info!("{} ensured virshle database.", "[init]".yellow(),);
        Ok(self)
    }
}
