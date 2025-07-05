/*
* This module is to connect to a virshle instance through local socket.
*/

use super::Stream;
use super::{Connection, ConnectionHandle, ConnectionState};
use super::{TcpUri, Uri};
use crate::cloud_hypervisor::Vm;
use crate::config::Node;

// Http
use http_body_util::{BodyExt, Full};
use hyper::body::{Body, Bytes, Incoming};
use hyper::client::conn::http1; // {handshake, SendRequest};
use hyper::client::conn::http2; // {handshake};
use hyper::{Request, Response as HyperResponse, StatusCode};
use hyper_util::rt::TokioIo;

use tokio::spawn;
use tokio::task::JoinHandle;

use serde::{Deserialize, Serialize};
use serde_json::{from_slice, Value};

// Socket
use std::path::Path;
use tokio::net::TcpStream;

// Error Handling
use log::{info, trace};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{ConnectionError, LibError, VirshleError, WrapError};

#[derive(Default)]
pub struct TcpConnection {
    pub uri: TcpUri,
}
impl ConnectionHandle for TcpConnection {
    async fn open(&mut self) -> Result<Stream, VirshleError> {
        let addrs = format!("{}:{}", self.uri.host, self.uri.port);
        let stream: TcpStream = match TcpStream::connect(&addrs).await {
            Err(e) => {
                let message = format!("Couldn't connect to tcp endpoint: {}", &addrs);
                let help = format!("Does the endpoint exists?");
                let err = ConnectionError::DaemonDown;
                return Err(err.into());
            }
            Ok(v) => v,
        };
        Ok(Stream::Tcp(stream))
    }
    /*
     * No need to close a stream as it is dropped once variable gets out of scope.
     */
    async fn close(&mut self) -> Result<(), VirshleError> {
        Ok(())
    }

    async fn get_state(&mut self) -> Result<ConnectionState, VirshleError> {
        let res = self.open().await;
        match res {
            Err(err) => match &err {
                VirshleError::ConnectionError(err) => match err {
                    ConnectionError::DaemonDown => Ok(ConnectionState::DaemonDown),
                    ConnectionError::SocketNotFound => Ok(ConnectionState::SocketNotFound),
                    _ => Ok(ConnectionState::Unreachable),
                },
                _ => Ok(ConnectionState::Unreachable),
            },
            Ok(conn) => Ok(ConnectionState::DaemonUp),
        }
    }
}
impl TcpConnection {
    pub fn new(url: &str) -> Result<Self, VirshleError> {
        let tcp_uri = TcpUri::new(url)?;
        Ok(Self {
            uri: tcp_uri,
            ..Default::default()
        })
    }
}
