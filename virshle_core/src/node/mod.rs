mod best;
mod display;
mod info;

use crate::config::{Config, VmTemplate};
pub use info::{HostCpu, HostDisk, HostInfo, HostRam, NodeInfo};

use std::string::ToString;

// Connection
use virshle_network::connection::{
    Connection, ConnectionHandle, SshConnection, TcpConnection, UnixConnection, Uri,
};

use serde::{Deserialize, Serialize};

// Error Handling
use miette::{Error, Result};
use tracing::{trace, warn};
use virshle_error::{LibError, VirshleError, WrapError};

///A declaration of a remote/local virshle daemon virshle nodes (alias and address)
/// to be queried by the cli.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Peer {
    pub alias: Option<String>,
    pub url: String,
    pub weight: Option<i32>,
    pub public_key: Option<String>,
}
impl Peer {
    pub fn alias(&self) -> Result<String, VirshleError> {
        let res = match self.alias.clone() {
            Some(v) => v,
            None => self.did()?,
        };
        Ok(res)
    }
    pub fn did(&self) -> Result<String, VirshleError> {
        Ok("did".to_string())
    }
}

impl Default for Peer {
    fn default() -> Self {
        // let url = "unix://".to_owned() + &NodeServer::get_socket().unwrap();
        let url = "unix:///var/lib/virshle/virshle.sock".to_owned();
        Self {
            alias: Some("default".to_owned()),
            url,
            weight: None,
            public_key: None,
        }
    }
}
impl Peer {
    pub fn new(alias: &str, url: &str) -> Result<Self, VirshleError> {
        let e = Peer {
            alias: Some(alias.to_owned()),
            url: url.to_owned(),
            weight: None,
            public_key: None,
        };
        Ok(e)
    }
}

impl TryInto<Connection> for Peer {
    type Error = VirshleError;
    fn try_into(self) -> Result<Connection, Self::Error> {
        (&self).try_into()
    }
}
impl TryInto<Connection> for &Peer {
    type Error = VirshleError;
    fn try_into(self) -> Result<Connection, Self::Error> {
        let conn = match Uri::new(&self.url)? {
            Uri::SshUri(v) => Connection::SshConnection(SshConnection {
                uri: v,
                ssh_handle: None,
            }),
            Uri::LocalUri(v) => Connection::UnixConnection(UnixConnection { uri: v }),
            Uri::TcpUri(v) => Connection::TcpConnection(TcpConnection { uri: v }),
        };
        Ok(conn)
    }
}

impl Peer {
    pub async fn unwrap_or_default(node_alias: Option<String>) -> Result<Peer, VirshleError> {
        let node = match node_alias {
            Some(node_alias) => match Peer::get_by_alias(&node_alias) {
                Ok(node) => node,
                Err(_) => Peer::default(),
            },
            None => Peer::default(),
        };
        Ok(node)
    }

    /*
     * Open connection to node and return handler.
     */
    pub async fn open(&self) -> Result<Connection, VirshleError> {
        let mut conn: Connection = self.try_into()?;
        match conn.open().await {
            Ok(v) => return Ok(conn),
            Err(e) => {
                warn!("{}", e);
                let message = format!(
                    "Couldn't connect to virshle daemon on node {:?}.",
                    self.alias
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
impl Peer {
    /// Returns nodes defined in configuration,
    /// plus the default local node.
    pub fn get_all() -> Result<Vec<Peer>, VirshleError> {
        let config = Config::get()?;
        let nodes = config.peers()?;
        Ok(nodes)
    }
    /// Returns node with alias.
    pub fn get_by_alias(alias: &str) -> Result<Peer, VirshleError> {
        let nodes: Vec<Peer> = Peer::get_all()?;
        let filtered_nodes: Vec<Peer> = nodes
            .iter()
            .filter(|e| e.alias().unwrap() == alias)
            .map(|e| e.to_owned())
            .collect();

        let node = filtered_nodes.first();
        match node {
            Some(node) => Ok(node.to_owned()),
            None => {
                let node_aliases: Vec<String> = nodes
                    .iter()
                    .map(|e| e.alias().unwrap().to_owned())
                    .collect();
                let node_aliases: String = node_aliases.join(",");
                let message = format!("Couldn't find node with alias: {:#?}", alias);
                let help = format!("Available nodes are:\n[{node_aliases}]");
                let err = LibError::builder().msg(&message).help(&help).build();
                return Err(err.into());
            }
        }
    }
}
