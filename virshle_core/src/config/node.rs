use crate::http_api::Server;
use crate::http_cli::{Connection, NodeConnection};
use std::fmt;
use url::Url;

use serde::{Deserialize, Serialize};
use users::{get_current_uid, get_user_by_uid};

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

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

impl Node {
    pub async fn open(&self) -> Result<NodeConnection, VirshleError> {
        let mut conn = NodeConnection::from(self);
        conn.open().await?;
        Ok(conn)
    }
}
