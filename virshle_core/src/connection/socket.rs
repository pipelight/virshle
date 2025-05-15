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
use hyper::client::conn::http1::{handshake, SendRequest};
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
    pub handle: Option<StreamHandle>,
    pub state: ConnectionState,
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

pub struct StreamHandle {
    pub sender: SendRequest<Full<Bytes>>,
    pub connection: JoinHandle<Result<(), hyper::Error>>,
}

impl ConnectionHandle for UnixConnection {
    async fn open(&mut self) -> Result<&mut Self, VirshleError> {
        let socket = &self.uri.path;
        let stream: TokioIo<UnixStream> = match UnixStream::connect(Path::new(&socket)).await {
            Err(e) => {
                let message = format!("Couldn't connect to socket: {}", socket);
                let help = format!("Does the socket exist?");
                let err = ConnectionError::SocketNotFound;
                return Err(err.into());
            }
            Ok(v) => TokioIo::new(v),
        };

        match handshake(stream).await {
            Err(e) => {
                let help = "Do you have the right credentials";
                let message = format!("Connection refused for socket: {socket}");
                let err = WrapError::builder()
                    .msg(&message)
                    .help(&help)
                    .origin(Error::from_err(e))
                    .build();
                return Err(err.into());
            }
            Ok((sender, connection)) => {
                info!("Connected to socket at {}", self.uri);
                self.handle = Some(StreamHandle {
                    sender,
                    connection: spawn(async move { connection.await }),
                });
            }
        };
        Ok(self)
    }
    /*
     * No need to close a stream as it is dropped once variable gets out of scope.
     */
    async fn close(&self) -> Result<(), VirshleError> {
        Ok(())
    }
}
