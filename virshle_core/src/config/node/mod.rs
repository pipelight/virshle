mod info;
pub use info::NodeInfo;

use crate::api::Server;
use crate::connection::{Connection, ConnectionHandle, NodeConnection, Uri};
use crate::http_request::HttpRequest;
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
        let url = "unix://".to_owned() + &Server::get_socket().unwrap();
        Self {
            name: "default".to_owned(),
            url,
        }
    }
}

impl Node {
    pub async fn get_info(&self) -> Result<NodeInfo, VirshleError> {
        let mut conn = self.open().await?;
        let info: NodeInfo = conn.get("/node/info").await?.to_value().await?;
        Ok(info)
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
