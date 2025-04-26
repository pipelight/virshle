/*
* This module is to connect to a virshle instance through local socket.
*/

use super::Response;
use super::{Connection, HttpRequest, NodeConnection};
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
use log::{debug, info};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

/// This struct is a convenience wrapper
/// around a unixsocket
pub struct UnixConnection {
    pub uri: LocalUri,
    pub handle: Option<StreamHandle>,
}
impl UnixConnection {
    pub fn new(path: &str) -> Self {
        Self {
            uri: LocalUri {
                path: path.to_owned(),
            },
            handle: None,
        }
    }
}

pub struct StreamHandle {
    pub sender: SendRequest<Full<Bytes>>,
    pub connection: JoinHandle<Result<(), hyper::Error>>,
}

impl Connection for UnixConnection {
    async fn open(&mut self) -> Result<&mut Self, VirshleError> {
        let socket = &self.uri.path;
        let stream: TokioIo<UnixStream> = match UnixStream::connect(Path::new(&socket)).await {
            Err(e) => {
                let help = format!("Does the following socket exist?\n{socket}");
                let err = WrapError::builder()
                    .msg("Couldn't connect to socket")
                    .help(&help)
                    .origin(Error::from_err(e))
                    .build();
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
                self.handle = Some(StreamHandle {
                    sender,
                    connection: spawn(async move { connection.await }),
                });
            }
        };
        Ok(self)
    }

    async fn send(
        &mut self,
        endpoint: &str,
        request: &Request<Full<Bytes>>,
    ) -> Result<Response, VirshleError> {
        if let Some(handle) = &mut self.handle {
            let response: HyperResponse<Incoming> =
                handle.sender.send_request(request.to_owned()).await?;

            let status: StatusCode = response.status();
            let response: Response = Response::new(endpoint, response);
            debug!("{:#?}", response);

            // if !status.is_success() {
            //     let message = format!("Status failed: {}", status);
            //     return Err(LibError::new(&message, "").into());
            // }

            Ok(response)
        } else {
            let err = LibError::new("Connection has no handler.", "open connection first.");
            return Err(err.into());
        }
    }
    // async fn close(socket: &str) -> Result<(), VirshleError> {
    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tokio::fs;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;
    use tokio::time::{sleep, Duration};

    /*
     * Create a socket and listens to incoming connections
     */
    async fn create_socket(socket: &str) -> Result<()> {
        let path = PathBuf::from(socket);
        let _ = tokio::fs::remove_file(&path).await;
        tokio::fs::create_dir_all(path.parent().unwrap())
            .await
            .unwrap();

        let listener = UnixListener::bind(socket).into_diagnostic()?;
        match listener.accept().await {
            Ok((mut socket, addr)) => {
                println!("Got a client: {:?} - {:?}", socket, addr);
                socket.write_all(b"hello world").await.into_diagnostic()?;
                let mut response = String::new();
                socket
                    .read_to_string(&mut response)
                    .await
                    .into_diagnostic()?;
                println!("{}", response);
            }
            Err(e) => println!("accept function failed: {:?}", e),
        }
        Ok(())
    }

    /*
     * Delete socket at specified path.
     * To be used after create_socket.
     */
    async fn remove_socket(socket: &str) -> Result<()> {
        fs::remove_file(socket).await.into_diagnostic()?;
        Ok(())
    }

    #[tokio::test]
    async fn try_send_local_request() -> Result<()> {
        let node = Node::default();
        let mut node_connection = NodeConnection::from(&node);

        let conn = node_connection.open().await?;
        conn.get("/vm/list").await?;

        Ok(())
    }
}
