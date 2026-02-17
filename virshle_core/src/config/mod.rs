mod definition;
mod load;
mod template;
mod user_data;

// Reexport
pub use template::{
    disk::DiskTemplate,
    vm::{NetType, VmNet, VmTemplate, VmTemplateTable},
    TemplateConfig,
};
pub use user_data::{Account, UserData};

use crate::database;
use crate::hypervisor::Vm;
use crate::network::{dhcp::DhcpType, ovs};
use crate::node::Node;

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::convert::Into;
use std::fs;
use std::path::Path;

// Error Handling
use log::{debug, info};
use miette::Result;
use virshle_error::{LibError, VirshleError};

pub const MANAGED_DIR: &'static str = "/var/lib/virshle";
pub const CONFIG_DIR: &'static str = "/etc/virshle";

// Nodes maximum staturation values in %.
pub const MAX_RAM_RESERVATION: f64 = 250_f64;
pub const MAX_CPU_RESERVATION: f64 = 300_f64;
pub const MAX_DISK_RESERVATION: f64 = 95_f64;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct NodeConfig {
    pub alias: Option<String>,
    pub private_key: String,
    pub public_key: String,
}
impl Into<Node> for NodeConfig {
    fn into(self) -> Node {
        (&self).into()
    }
}
impl Into<Node> for &NodeConfig {
    fn into(self) -> Node {
        Node {
            alias: self.alias.clone(),
            url: "".to_owned(),
            weight: None,
            public_key: Some(self.public_key.clone()),
        }
    }
}

/// The main virshle cli and daemon configuration struct.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    // Server
    /// The local node configuration
    pub node: Option<NodeConfig>,
    /// Vm templates
    pub template: Option<TemplateConfig>,
    /// Network configuration
    pub dhcp: Option<DhcpType>,

    // Client
    /// List of remote node
    nodes: Option<Vec<Node>>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            node: None,
            nodes: Some(vec![Node::default()]),
            dhcp: None,
            template: None,
        }
    }
}
impl Config {
    /// Ensure virshle resources:
    ///   - a clean working directory and database.
    ///   - an initial configuration.
    ///   - a dedicated network virtual switch.
    pub async fn ensure_all() -> Result<(), VirshleError> {
        Self::ensure_directories().await?;
        Self::ensure_database().await?;
        Self::ensure_network().await?;

        Self::_clean_directories().await?;
        Self::_clean_leases().await?;
        Ok(())
    }
    /// Ensure virshle working directories exists.
    pub async fn ensure_directories() -> Result<(), VirshleError> {
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
            }
        }
        info!("{} created virshle filetree.", "[init]".yellow(),);
        Ok(())
    }
    /// Clean orphan vm files if vm not in database.
    pub async fn _clean_directories() -> Result<(), VirshleError> {
        let vms = Vm::database().await?.many().get().await?;
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
    /// Clean dhcp leases
    pub async fn _clean_leases() -> Result<(), VirshleError> {
        match Config::get()?.dhcp {
            Some(DhcpType::Kea(kea_dhcp)) => {
                kea_dhcp.clean_leases().await?;
            }
            _ => {}
        };
        info!("{} delete unused leases", "[kea-dhcp]".yellow(),);
        Ok(())
    }
}

// Getters
impl Config {
    pub fn nodes(&self) -> Result<Vec<Node>, VirshleError> {
        let nodes: Vec<Node> = match &self.nodes {
            Some(node) => node.to_owned(),
            None => vec![Node::default()],
        };
        Ok(nodes)
    }
    pub fn get_templates(&self) -> Result<Vec<VmTemplate>, VirshleError> {
        if let Some(template) = &self.template {
            if let Some(vm) = &template.vm {
                return Ok(vm.to_owned());
            }
        }
        Ok(vec![])
    }
    pub fn get_template_by_name(&self, name: &str) -> Result<VmTemplate, VirshleError> {
        let templates = self.get_templates()?;
        let res = templates.iter().find(|e| e.name == name);
        match res {
            Some(res) => Ok(res.to_owned()),
            None => {
                let message = format!("Couldn't find template {:#?}", name);
                let templates_name = templates
                    .iter()
                    .map(|e| e.name.to_owned())
                    .collect::<Vec<String>>()
                    .join(",");
                let help = format!("Available templates are:\n[{templates_name}]");
                let err = LibError::builder().msg(&message).help(&help).build();
                Err(err.into())
            }
        }
    }
}
