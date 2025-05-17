/*
* Cloud hypervisor compatibility layer.
*
* This crate is an api to easily connect to multiple endpoints and send/receive data streams.
*
* - Local unix sockets
* - unix sockets behind ssh.
*
* It is combined with the HttpRequest trait to send/receive http between enpoints
* with trivial methods like .get(), put(json(data)).
*
* Sources:
* https://levelup.gitconnected.com/learning-rust-http-via-unix-socket-fee3241b4340
* https://github.com/amacal/etl0/blob/85d155b1cdf2f7962188cd8b8833442a1e6a1132/src/etl0/src/docker/http.rs
* https://docs.rs/hyperlocal/latest/hyperlocal/
*/

// Main connection types.

mod node;

mod socket;
mod ssh;
mod uri;

// Reexport
pub use socket::UnixConnection;
pub use ssh::SshConnection;
pub use uri::{LocalUri, SshUri, Uri};

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::Request;

use serde::{Deserialize, Serialize};
use std::future::Future;

// Error Handling
use miette::{Error, Result};
use virshle_error::{ConnectionError, VirshleError, WrapError};

pub trait ConnectionHandle {
    /*
     * Open connection to
     * - unix socket
     * - or ssh then unix socket
     */
    fn open(&mut self) -> impl Future<Output = Result<&mut Self, VirshleError>> + Send;
    /*
     * Close connection
     */
    fn close(&self) -> impl Future<Output = Result<(), VirshleError>> + Send;
    /*
     * Get connection state
     */
    fn get_state(&self) -> Result<ConnectionState, VirshleError>;
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ConnectionState {
    /// Success: Connection established and daemon is up!
    DaemonUp,

    /// Uninitialized: Connection not established.
    #[default]
    Down,

    // Warning: Small error
    SshAuthError,

    // Error
    DaemonDown,
    SocketNotFound,
    /// Unknown network reason.
    Unreachable,
}

pub enum Connection {
    SshConnection(SshConnection),
    UnixConnection(UnixConnection),
}
impl ConnectionHandle for Connection {
    async fn open(&mut self) -> Result<&mut Self, VirshleError> {
        match self {
            Connection::SshConnection(connection) => {
                let res = connection.open().await;
                match res {
                    Err(err) => {
                        match &err {
                            VirshleError::ConnectionError(err) => match err {
                                ConnectionError::DaemonDown => {
                                    connection.state = ConnectionState::DaemonDown;
                                }
                                ConnectionError::SocketNotFound => {
                                    connection.state = ConnectionState::SocketNotFound;
                                }
                                ConnectionError::SshAuthError
                                | ConnectionError::RusshError(_)
                                | ConnectionError::SshKeyError(_)
                                | ConnectionError::SshAgentError(_) => {
                                    connection.state = ConnectionState::SshAuthError;
                                }
                            },
                            _ => {
                                connection.state = ConnectionState::Unreachable;
                            }
                        };
                        return Err(err);
                    }
                    Ok(conn) => {
                        conn.state = ConnectionState::DaemonUp;
                        Ok(self)
                    }
                }
            }
            Connection::UnixConnection(connection) => {
                let res = connection.open().await;
                match res {
                    Err(err) => {
                        match &err {
                            VirshleError::ConnectionError(err) => match err {
                                ConnectionError::DaemonDown => {
                                    connection.state = ConnectionState::DaemonDown;
                                }
                                ConnectionError::SocketNotFound => {
                                    connection.state = ConnectionState::SocketNotFound;
                                }
                                _ => {
                                    connection.state = ConnectionState::Unreachable;
                                }
                            },
                            _ => {
                                connection.state = ConnectionState::Unreachable;

                                let help = "Do you have the right credentials";
                                let message = format!("Connection refused for");
                                let err = WrapError::builder()
                                    .msg(&message)
                                    .help(&help)
                                    .origin(Error::from_err(err))
                                    .build();
                                return Err(err.into());
                            }
                        }
                        return Err(err);
                    }
                    Ok(conn) => {
                        conn.state = ConnectionState::DaemonUp;
                        Ok(self)
                    }
                }
            }
        }
    }
    async fn close(&self) -> Result<(), VirshleError> {
        match self {
            Connection::SshConnection(ssh_connection) => {
                let _ = ssh_connection.close().await?;
            }
            _ => {}
        };
        Ok(())
    }
    fn get_state(&self) -> Result<ConnectionState, VirshleError> {
        match self {
            Connection::SshConnection(connection) => Ok(connection.state.to_owned()),
            Connection::UnixConnection(connection) => Ok(connection.state.to_owned()),
        }
    }
}

pub struct NodeConnection(pub Connection);
impl ConnectionHandle for NodeConnection {
    async fn close(&self) -> Result<(), VirshleError> {
        self.0.close().await?;
        Ok(())
    }
    async fn open(&mut self) -> Result<&mut Self, VirshleError> {
        self.0.open().await?;
        Ok(self)
    }
    fn get_state(&self) -> Result<ConnectionState, VirshleError> {
        let state = self.0.get_state()?;
        Ok(state)
    }
}

pub struct VmConnection(pub Connection);
impl ConnectionHandle for VmConnection {
    async fn close(&self) -> Result<(), VirshleError> {
        self.0.close().await?;
        Ok(())
    }
    async fn open(&mut self) -> Result<&mut Self, VirshleError> {
        self.0.open().await?;
        Ok(self)
    }
    fn get_state(&self) -> Result<ConnectionState, VirshleError> {
        let state = self.0.get_state()?;
        Ok(state)
    }
}
