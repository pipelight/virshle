/*
* Cloud hypervisor compatibility layer.
*
* This crate is an api to connect to socket and send http requests.
* Sockets may be local or over ssh.
*
* Sources:
* https://levelup.gitconnected.com/learning-rust-http-via-unix-socket-fee3241b4340
* https://github.com/amacal/etl0/blob/85d155b1cdf2f7962188cd8b8833442a1e6a1132/src/etl0/src/docker/http.rs
* https://docs.rs/hyperlocal/latest/hyperlocal/
*/

mod http_request;
mod socket;
mod ssh;
mod uri;

use crate::config::Node;
use std::future::Future;

// Http
use http_body_util::{BodyExt, Full};
pub use http_request::{HttpRequest, Response};
use hyper::body::{Body, Bytes, Incoming};
pub use hyper::Request;

pub use socket::UnixConnection;
pub use ssh::SshConnection;
pub use uri::{LocalUri, SshUri, Uri};

// Error Handling
use log::info;
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub trait Connection {
    /*
     * Open connection to
     * - unix socket
     * - or ssh then unix socket
     */
    fn open(&mut self) -> impl Future<Output = Result<&mut Self, VirshleError>> + Send;
    /*
     * Send the http request.
     * Internally used by get(), post() and put() methods.
     */
    fn send(
        &mut self,
        endpoint: &str,
        request: &Request<Full<Bytes>>,
    ) -> impl Future<Output = Result<Response, VirshleError>> + Send;
    /*
     * Close connection
     */
    // fn close(&self) -> Result<(), VirshleError>;
}

pub type VmConnection = UnixConnection;

pub enum NodeConnection {
    SshConnection(SshConnection),
    UnixConnection(UnixConnection),
    VmConnection(VmConnection),
}

impl Connection for NodeConnection {
    async fn open(&mut self) -> Result<&mut Self, VirshleError> {
        match self {
            NodeConnection::SshConnection(ssh_connection) => {
                let _ = ssh_connection.open().await?;
            }
            NodeConnection::UnixConnection(unix_connection) => {
                let _ = unix_connection.open().await?;
            }
            NodeConnection::VmConnection(vm_connection) => {
                let _ = vm_connection.open().await?;
            }
        };
        Ok(self)
    }
    async fn send(
        &mut self,
        endpoint: &str,
        request: &Request<Full<Bytes>>,
    ) -> Result<Response, VirshleError> {
        match self {
            NodeConnection::SshConnection(ssh_connection) => {
                let response = ssh_connection.open().await?.send(endpoint, request).await?;
                return Ok(response);
            }
            NodeConnection::UnixConnection(unix_connection) => {
                let response = unix_connection
                    .open()
                    .await?
                    .send(endpoint, request)
                    .await?;
                return Ok(response);
            }
            NodeConnection::VmConnection(vm_connection) => {
                let response = vm_connection.open().await?.send(endpoint, request).await?;
                return Ok(response);
            }
        };
    }
}

impl From<&Node> for NodeConnection {
    fn from(e: &Node) -> Self {
        let uri = Uri::new(&e.url).unwrap();
        match uri {
            Uri::SshUri(ssh_uri) => NodeConnection::SshConnection(SshConnection {
                uri: ssh_uri,
                ssh_handle: None,
                handle: None,
            }),
            Uri::LocalUri(unix_uri) => NodeConnection::UnixConnection(UnixConnection {
                uri: unix_uri,
                handle: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_to_connection() -> Result<()> {
        let node = Node::default();
        let res = NodeConnection::from(&node);
        Ok(())
    }
}
