use crate::config::Node;

// Http
use http_body_util::{BodyExt, Full};
// pub use http_request::{HttpRequest, Response};
use hyper::body::{Body, Bytes, Incoming};
pub use hyper::Request;

use super::uri::{LocalUri, SshUri, Uri};
use super::ConnectionHandle;
use super::{Connection, NodeConnection, SshConnection, UnixConnection};

use serde::{Deserialize, Serialize};

// Error Handling
use log::{info, warn};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

impl From<&Node> for NodeConnection {
    fn from(e: &Node) -> Self {
        let uri = Uri::new(&e.url).unwrap();
        match uri {
            Uri::SshUri(ssh_uri) => NodeConnection(Connection::SshConnection(SshConnection {
                uri: ssh_uri,
                ..Default::default()
            })),
            Uri::LocalUri(unix_uri) => NodeConnection(Connection::UnixConnection(UnixConnection {
                uri: unix_uri,
                ..Default::default()
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_local_node_to_connection() -> Result<()> {
        let node = Node::default();
        let res = NodeConnection::from(&node);
        Ok(())
    }
    #[test]
    fn test_remote_node_to_connection() -> Result<()> {
        let node = Node {
            url: "ssh://localhost/var/lib/virshle/virshle.sock".to_owned(),
            name: "test".to_owned(),
        };
        let res = NodeConnection::from(&node);
        Ok(())
    }
}
