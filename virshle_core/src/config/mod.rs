mod definition;
mod load;
mod node;
mod template;
mod user_data;
/// Initialize system directories, network, database...
pub mod init;

// Reexport
pub use definition::Definition;
use load::PreConfig;
pub use node::{Node, NodeConfig};
pub use template::{
    disk::DiskTemplate,
    vm::{NetType, VmNet, VmTemplate, VmTemplateTable},
    TemplateConfig,
};
pub use user_data::{Account, SshParams, User, UserData};

use crate::network::{dhcp::DhcpType, ovs};
use crate::peer::Peer;

use bon::bon;
use indexmap::IndexMap;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::convert::Into;
use std::fs;
use std::path::Path;

// Error Handling
use log::{debug, info};
use miette::Result;
use virshle_error::{LibError, VirshleError};

// Nodes maximum staturation values in %.
pub const MAX_RAM_RESERVATION: f64 = 250_f64;
pub const MAX_CPU_RESERVATION: f64 = 300_f64;
pub const MAX_DISK_RESERVATION: f64 = 95_f64;

/// The main virshle cli and daemon configuration struct.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    // Server
    /// The local node configuration
    pub node: Node,
    /// Vm templates
    pub templates: IndexMap<String, VmTemplate>,
    /// Network configuration
    pub dhcp: Option<DhcpType>,

    // Client
    /// List of remote node
    peers: IndexMap<String, Peer>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            node: Node::default(),
            peers: IndexMap::new(),
            templates: IndexMap::new(),
            dhcp: None,
        }
    }
}
// Getters
#[bon]
impl Config {
    pub fn get() -> Result<Config, VirshleError> {
        let res: Config = PreConfig::get()?.try_into()?;
        Ok(res)
    }
    // Return self node AND remote nodes.
    pub fn peers(&self) -> Result<IndexMap<String, Peer>, VirshleError> {
        let mut node_and_peers: IndexMap<String, Peer> = IndexMap::new();
        if !self.node.passive {
            let node: Peer = (&self.node).into();
            node_and_peers.insert(node.alias.clone(), node.clone());
        }
        node_and_peers.extend(self.peers.to_owned());
        Ok(node_and_peers)
    }
    /// Returns node with alias.
    #[builder(
        finish_fn = get, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub fn peer(&self, alias: Option<String>) -> Result<Peer, VirshleError> {
        let peers = self.peers()?;
        match alias {
            Some(alias) => {
                match peers.get(&alias) {
                    Some(v) => Ok(v.to_owned()),
                    None => {
                        let aliases: Vec<String> = peers.into_keys().collect();
                        let aliases: String = aliases.join(",");

                        let message = format!("Couldn't find node with alias: {:#?}", alias);
                        let help = format!("Available nodes are:\n[{aliases}]");
                        let err = LibError::builder().msg(&message).help(&help).build();
                        return Err(err.into());
                    }
                }
            }
            None => Ok(self.node.clone().into()),
        }
    }
    /// Get template by name.
    pub fn template(&self, name: &str) -> Result<VmTemplate, VirshleError> {
        let res = self.templates.get(name);
        match res {
            Some(res) => Ok(res.to_owned()),
            None => {
                let message = format!("Couldn't find template {:#?}", name);
                let templates_name = self
                    .templates
                    .iter()
                    .map(|(name, _)| name.to_owned())
                    .collect::<Vec<String>>()
                    .join(",");
                let help = format!("Available templates are:\n[{templates_name}]");
                let err = LibError::builder().msg(&message).help(&help).build();
                Err(err.into())
            }
        }
    }
}
