mod best;
pub mod info;
pub use info::NodeInfo;

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
}

impl Default for Node {
    fn default() -> Self {
        let url = "unix://".to_owned() + &NodeServer::get_socket().unwrap();
        Self {
            name: "default".to_owned(),
            url,
        }
    }
}
impl Node {
    pub fn new(name: &str, url: &str) -> Result<Self, VirshleError> {
        let e = Node {
            name: name.to_owned(),
            url: url.to_owned(),
        };
        Ok(e)
    }
}

impl Node {
    pub async fn unwrap_or_default(node_name: Option<String>) -> Result<Node, VirshleError> {
        let config = VirshleConfig::get()?;
        let node = match node_name {
            Some(node_name) => match config.get_node_by_name(&node_name) {
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
