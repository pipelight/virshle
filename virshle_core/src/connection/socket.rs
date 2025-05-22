/*
* This module is to connect to a virshle instance through local socket.
*/

use super::{Connection, ConnectionHandle, ConnectionState, NodeConnection};
use super::{LocalUri, Uri};
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
use tokio::net::UnixStream;

// Error Handling
use log::{info, trace};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{ConnectionError, LibError, VirshleError, WrapError};

/// This struct is a convenience wrapper
/// around a unixsocket
#[derive(Default)]
pub struct UnixConnection {
    pub uri: LocalUri,
    pub stream: Option<UnixStream>,
}
impl UnixConnection {
    pub fn new(path: &str) -> Self {
        Self {
            uri: LocalUri {
                path: path.to_owned(),
            },
            ..Default::default()
        }
    }
}

impl ConnectionHandle for UnixConnection {
    async fn open(&mut self) -> Result<&mut Self, VirshleError> {
        let socket = &self.uri.path;
        if !Path::new(socket).exists() {
            let err = ConnectionError::SocketNotFound;
            return Err(err.into());
        }

        let stream: UnixStream = match UnixStream::connect(Path::new(&socket)).await {
            Err(e) => {
                let message = format!("Couldn't connect to socket: {}", socket);
                let help = format!("Does the socket exist?");
                let err = ConnectionError::DaemonDown;
                return Err(err.into());
            }
            // Ok(v) => TokioIo::new(v),
            Ok(v) => v,
        };
        self.stream = Some(stream);

        Ok(self)
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

    fn get_stream<T>(&self) -> Result<T, VirshleError>
    where
        T: tokio::io::AsyncRead + tokio::io::AsyncWrite,
    {
        Ok(self.stream)
    }
}
