mod best;
pub mod info;
pub use info::NodeInfo;

use owo_colors::OwoColorize;

use crate::api::NodeServer;
use crate::connection::{Connection, ConnectionHandle, Uri};
use crate::http_request::{Rest, RestClient};
use crate::Vm;

// Http
use std::fmt;
use url::Url;

use serde::{Deserialize, Serialize};
use users::{get_current_uid, get_user_by_uid};

// Error Handling
use log::{info, warn};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

use super::VirshleConfig;

/*
* A declaration of a remote/local virshle daemon virshle nodes (name and address)
* to be queried by the cli.
*/
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Node {
    pub name: String,
    pub url: String,
    pub weight: i32,
}

impl Default for Node {
    fn default() -> Self {
        let url = "unix://".to_owned() + &NodeServer::get_socket().unwrap();
        Self {
            name: "default".to_owned(),
            url,
            weight: 0,
        }
    }
}
impl Node {
    pub fn new(name: &str, url: &str) -> Result<Self, VirshleError> {
        let e = Node {
            name: name.to_owned(),
            url: url.to_owned(),
            weight: 0,
        };
        Ok(e)
    }
}

impl Node {
    pub async fn unwrap_or_default(node_name: Option<String>) -> Result<Node, VirshleError> {
        let node = match node_name {
            Some(node_name) => match Node::get_by_name(&node_name) {
                Ok(node) => node,
                Err(_) => Node::default(),
            },
            None => Node::default(),
        };
        Ok(node)
    }

    /*
     * Open connection to node and return handler.
     */
    pub async fn open(&self) -> Result<Connection, VirshleError> {
        let mut conn = Connection::from(self);
        match conn.open().await {
            Ok(v) => return Ok(conn),
            Err(e) => {
                warn!("{}", e);
                let message = format!(
                    "Couldn't connect to virshle daemon on node {:?}.",
                    self.name
                );
                let help = format!("Is virshle daemon running at url: {:?} ?", self.url);
                let err = WrapError::builder()
                    .msg(&message)
                    .help(&help)
                    .origin(Error::from_err(e))
                    .build();
                return Err(err.into());
            }
        };
    }
}
impl Node {
    pub fn get_header(&self) -> Result<String, VirshleError> {
        let name = self.name.bright_purple().bold().to_string();
        let header: String = match Uri::new(&self.url)? {
            Uri::SshUri(e) => format!(
                "{name} on {}@{}",
                e.user.yellow().bold(),
                e.host.green().bold()
            ),
            Uri::LocalUri(e) => format!("{name} on {}", "localhost".green().bold()),
            Uri::TcpUri(e) => format!(
                "{name} on {}{}",
                e.host.green().bold(),
                e.port.blue().bold()
            ),
        };
        Ok(header)
    }
    /// Returns nodes defined in configuration,
    /// plus the default local node.
    pub fn get_all() -> Result<Vec<Node>, VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes: Vec<Node> = match &config.node {
            Some(node) => node.to_owned(),
            None => vec![Node::default()],
        };
        Ok(nodes)
    }
    /// Returns node with name.
    pub fn get_by_name(name: &str) -> Result<Node, VirshleError> {
        let nodes: Vec<Node> = Node::get_all()?;
        let filtered_nodes: Vec<Node> = nodes
            .iter()
            .filter(|e| e.name == name)
            .map(|e| e.to_owned())
            .collect();

        let node = filtered_nodes.first();
        match node {
            Some(node) => Ok(node.to_owned()),
            None => {
                let node_names: Vec<String> = nodes.iter().map(|e| e.name.to_owned()).collect();
                let node_names: String = node_names.join(",");
                let message = format!("couldn't find node with name: {:#?}", name);
                let help = format!("Available nodes are:\n[{node_names}]");
                let err = LibError::builder().msg(&message).help(&help).build();
                return Err(err.into());
            }
        }
    }
}
