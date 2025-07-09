pub mod cache;
pub mod getters;
pub mod load;
pub mod node;

// Reexport
pub use node::{Node, NodeInfo};

use crate::api::NodeServer;
use crate::cloud_hypervisor::{Template, Vm, VmTemplate};
use crate::database;
use crate::network::dhcp::DhcpType;
use crate::network::ovs;

use owo_colors::OwoColorize;

// Global vars
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

// Config
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Error Handling
use log::{debug, info};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{CastError, LibError, TomlError, VirshleError, WrapError};

pub const MANAGED_DIR: &'static str = "/var/lib/virshle";
pub const CONFIG_DIR: &'static str = "/etc/virshle";

/*
* The main virshle cli and daemon configuration struct.
*/
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VirshleConfig {
    node: Option<Vec<Node>>,
    pub template: Option<Template>,
    pub dhcp: Option<DhcpType>,
}
impl Default for VirshleConfig {
    fn default() -> Self {
        Self {
            node: Some(vec![Node::default()]),
            template: None,
            dhcp: None,
        }
    }
}
impl VirshleConfig {
    /// Ensure virshle resources:
    ///   - a clean working directory and database.
    ///   - an initial configuration.
    ///   - a dedicated network virtual switch.
    pub async fn ensure_all() -> Result<(), VirshleError> {
        Self::ensure_directories();
        Self::ensure_database();
        Self::ensure_network();

        Self::_clean_directories();
        Ok(())
    }
    /// Ensure virshle working directories exists.
    pub async fn ensure_directories() -> Result<(), VirshleError> {
        // Create storage/config directories
        let directories = [
            MANAGED_DIR.to_owned(),
            MANAGED_DIR.to_owned() + "/vm",
            CONFIG_DIR.to_owned(),
        ];
        for directory in directories {
            let path = Path::new(&directory);
            if !path.exists() {
                fs::create_dir_all(&directory)?;
            }
        }
        info!("{} created virshle filetree.", "[init]".yellow(),);
        Ok(())
    }
    /// Clean orphan vm files if vm not in database.
    pub async fn _clean_directories() -> Result<(), VirshleError> {
        let vms = Vm::get_all().await?;
        let uuids: Vec<String> = vms.iter().map(|e| e.uuid.to_string()).collect();

        let path = format!("{MANAGED_DIR}/vm");
        let path = Path::new(&path);
        for entry in path.read_dir()? {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    if !uuids.contains(&entry.file_name().to_str().unwrap().to_owned()) {
                        fs::remove_dir_all(entry.path())?;
                    }
                }
            }
        }
        debug!("Cleaned virshle filetree.");
        Ok(())
    }
    pub async fn ensure_network() -> Result<(), VirshleError> {
        ovs::ensure_switches().await?;
        info!(
            "{} created virshle ovs network configuration.",
            "[init]".yellow(),
        );
        Ok(())
    }
    pub async fn ensure_database() -> Result<(), VirshleError> {
        database::connect_or_fresh_db().await?;
        info!("{} ensured virshle database.", "[init]".yellow(),);
        Ok(())
    }
}
