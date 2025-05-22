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
mod state;

mod socket;
mod ssh;
mod uri;

// Reexport
pub use socket::UnixConnection;
pub use ssh::SshConnection;
pub use state::ConnectionState;
pub use uri::{LocalUri, SshUri, Uri};

// Http
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::client::conn::http1; // {handshake, SendRequest};
use hyper::client::conn::http2; // {handshake};
use hyper::Request;

// Async/Await
use tokio::task::JoinHandle;

use serde::{Deserialize, Serialize};
use std::future::Future;

// Error Handling
use miette::{Error, Result};
use virshle_error::{ConnectionError, VirshleError, WrapError};

pub trait ConnectionHandle {
    fn open(&mut self) -> impl Future<Output = Result<&mut Self, VirshleError>> + Send;
    fn close(&mut self) -> impl Future<Output = Result<(), VirshleError>> + Send;
    fn get_state(&mut self) -> impl Future<Output = Result<ConnectionState, VirshleError>> + Send;
    fn get_stream<T>(&self) -> Result<T, VirshleError>
    where
        T: tokio::io::AsyncRead + tokio::io::AsyncWrite;
}

pub enum Connection {
    SshConnection(SshConnection),
    UnixConnection(UnixConnection),
}
impl ConnectionHandle for Connection {
    async fn open(&mut self) -> Result<&mut Self, VirshleError> {
        match self {
            Connection::SshConnection(e) => {
                e.open().await?;
            }
            Connection::UnixConnection(e) => {
                e.open().await?;
            }
        };
        Ok(self)
    }
    async fn close(&mut self) -> Result<(), VirshleError> {
        match self {
            Connection::SshConnection(e) => {
                e.close().await?;
            }
            Connection::UnixConnection(e) => {
                e.close().await?;
            }
        };
        Ok(())
    }
    async fn get_state(&mut self) -> Result<ConnectionState, VirshleError> {
        match self {
            Connection::SshConnection(e) => e.get_state().await,
            Connection::UnixConnection(e) => e.get_state().await,
        }
    }
    fn get_stream<T>(&self) -> Result<T, VirshleError> {
        match self {
            Connection::SshConnection(e) => e.get_stream(),
            Connection::UnixConnection(e) => e.get_steam(),
        }
    }
}

pub struct NodeConnection(pub Connection);
impl ConnectionHandle for NodeConnection {
    async fn open(&mut self) -> Result<&mut Self, VirshleError> {
        self.0.open().await?;
        Ok(self)
    }
    async fn close(&mut self) -> Result<(), VirshleError> {
        self.0.close().await
    }
    async fn get_state(&mut self) -> Result<ConnectionState, VirshleError> {
        self.0.get_state().await
    }
    fn get_stream(&self) -> Result<ConnectionState, VirshleError> {
        self.0.get_stream()
    }
}

pub struct VmConnection(pub Connection);
impl ConnectionHandle for VmConnection {
    async fn open(&mut self) -> Result<&mut Self, VirshleError> {
        self.0.open().await?;
        Ok(self)
    }
    async fn close(&mut self) -> Result<(), VirshleError> {
        self.0.close().await?;
        Ok(())
    }
    async fn get_state(&mut self) -> Result<ConnectionState, VirshleError> {
        self.0.get_state().await
    }
    fn get_stream(&self) -> Result<ConnectionState, VirshleError> {
        self.0.get_stream()
    }
}
