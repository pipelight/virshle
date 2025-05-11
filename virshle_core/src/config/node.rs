use crate::http_api::{Host, Server};

use crate::http_cli::{Connection, HttpRequest, NodeConnection, Uri};
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

#[derive(Debug, Clone, Deserialize, Serialize, Hash, PartialEq, Eq)]
pub struct Node {
    pub name: String,
    pub url: String,
}
impl Default for Node {
    fn default() -> Self {
        let url = "unix://".to_owned() + &Server::get_socket().unwrap();
        Self {
            name: "default".to_owned(),
            url,
        }
    }
}
#[derive(Default, Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum NodeState {
    Running,
    #[default]
    Unreachable,
}

impl Node {
    pub async fn get_info(&self) -> Result<Host, VirshleError> {
        let mut conn = self.open().await?;
        let info: Host = conn.get("/node/info").await?.to_value().await?;
        Ok(info)
    }
    pub async fn get_num_vm(&self) -> Result<u64, VirshleError> {
        let mut conn = self.open().await?;
        let vms: Vec<Vm> = conn.get("/vm/list").await?.to_value().await?;
        let n = vms.len() as u64;
        Ok(n)
    }
    /*
     * Get node state.
     */
    pub async fn get_state(&self) -> Result<NodeState, VirshleError> {
        let state = match self.open().await {
            Err(e) => {
                warn!("{}", e);
                NodeState::Unreachable
            }
            Ok(conn) => {
                conn.close().await?;
                NodeState::Running
            }
        };
        Ok(state)
    }
    /*
     * Return connection handler.
     */
    pub fn get_connection(&self) -> Result<NodeConnection, VirshleError> {
        let conn = NodeConnection::from(self);
        Ok(conn)
    }
    /*
     * Open connection to node and return handler.
     */
    pub async fn open(&self) -> Result<NodeConnection, VirshleError> {
        let mut conn = NodeConnection::from(self);
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
