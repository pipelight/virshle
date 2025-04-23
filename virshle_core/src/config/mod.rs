pub mod cache;
pub mod load;
pub mod uri;

use crate::cloud_hypervisor::{Template, Vm, VmTemplate};
use crate::database;
use crate::network::Ovs;

// Global vars
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

// Config
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Error Handling
use log::info;
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{CastError, LibError, TomlError, VirshleError, WrapError};

pub const MANAGED_DIR: &'static str = "/var/lib/virshle";
pub const CONFIG_DIR: &'static str = "/etc/virshle";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VirshleConfig {
    pub connect: Option<Vec<Node>>,
    pub template: Option<Template>,
}
impl Default for VirshleConfig {
    fn default() -> Self {
        Self {
            connect: Some(vec![Node::default()]),
            template: None,
        }
    }
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Node {
    pub name: String,
    pub url: String,
}
impl Default for Node {
    fn default() -> Self {
        let url = "file://".to_owned() + MANAGED_DIR + "/virshle.sock";
        Self {
            name: "default".to_owned(),
            url,
        }
    }
}
impl VirshleConfig {
    pub async fn _clean_filetree() -> Result<(), VirshleError> {
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
        Ok(())
    }
    pub fn ensure_filetree() -> Result<(), VirshleError> {
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
        Ok(())
    }
    /*
     * Ensure virshle directories and configuration exists.
     */
    pub async fn init() -> Result<(), VirshleError> {
        Self::ensure_filetree();

        // Ensure vm database
        database::connect_db().await?;

        // Remove vm files that do not match any db entry
        Self::_clean_filetree().await?;

        // Ensure host and vm switches configuration
        Ovs::ensure_switches().await?;

        // TODO():
        // Create virshle daemon socket (for API calls)

        Ok(())
    }
}
