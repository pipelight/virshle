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
use crate::node::Peer;

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::convert::Into;
use std::fs;
use std::path::Path;
// Ssh
use rand_core::OsRng;
// use rand::rngs::OsRng;
use russh::keys::{ssh_key::Algorithm, PrivateKey, PublicKey};

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

/// The main virshle cli and daemon configuration struct.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    // Server
    /// The local node configuration
    node: Option<NodeConfig>,
    /// Vm templates
    pub template: Option<TemplateConfig>,
    /// Network configuration
    pub dhcp: Option<DhcpType>,

    // Client
    /// List of remote node
    peer: Option<Vec<Peer>>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            peer: Some(vec![Peer::default()]),

            node: None,
            template: None,
            dhcp: None,
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
    pub fn node(&self) -> Result<Node, VirshleError> {
        let res = match &self.node {
            Some(v) => v.try_into()?,
            None => Node::default(),
        };
        Ok(res)
    }
    pub fn peers(&self) -> Result<Vec<Peer>, VirshleError> {
        let peers: Vec<Peer> = match &self.peer {
            Some(peer) => peer.to_owned(),
            None => vec![Peer::default()],
        };
        Ok(peers)
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct NodeConfig {
    pub alias: Option<String>,
    pub private_key: String,
    pub public_key: String,
    pub passive: Option<bool>,
}
impl TryInto<Node> for NodeConfig {
    type Error = VirshleError;
    fn try_into(self) -> Result<Node, Self::Error> {
        (&self).try_into()
    }
}
impl TryInto<Node> for &NodeConfig {
    type Error = VirshleError;
    fn try_into(self) -> Result<Node, Self::Error> {
        let private_key = fs::read_to_string(&self.private_key)?;
        let public_key = fs::read_to_string(&self.public_key)?;
        Ok(Node {
            alias: Some("Self".to_owned()),
            private_key,
            public_key,
            passive: false,
        })
    }
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Node {
    pub alias: Option<String>,
    pub private_key: String,
    pub public_key: String,
    pub passive: bool,
}
impl Default for Node {
    fn default() -> Self {
        let key_pair = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let public_key = key_pair.public_key().to_openssh().unwrap();
        let private_key = key_pair
            .to_openssh(russh::keys::ssh_key::LineEnding::LF)
            .unwrap()
            .to_string();
        Node {
            alias: Some("Self".to_owned()),
            private_key,
            public_key,
            passive: false,
        }
    }
}

impl Into<Peer> for Node {
    fn into(self) -> Peer {
        (&self).into()
    }
}
impl Into<Peer> for &Node {
    fn into(self) -> Peer {
        Peer {
            alias: self.alias.clone(),
            url: "".to_owned(),
            weight: None,
            public_key: Some(self.public_key.clone()),
        }
    }
}
