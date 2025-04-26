/*
* This module is to connect to a virshle instance through ssh.
*/

use super::Response;
use super::{Connection, NodeConnection};
use super::{SshUri, Uri};
use crate::config::Node;
use crate::http_api::Server;

use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, Value};

// Ssh
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWrite, AsyncWriteExt};
use tokio::net::ToSocketAddrs;
use tokio::net::UnixStream;

use russh::client::{connect, Config, Handle as SshHandle, Handler, Msg};
use russh::keys::agent::client::AgentClient;
use russh::{
    keys::load_secret_key,
    keys::{ssh_key::Algorithm, PrivateKeyWithHashAlg, PublicKey},
    ChannelMsg, ChannelStream, CryptoVec, Disconnect,
};
use std::net::TcpStream;
use std::sync::Arc;

// Http
use super::socket::StreamHandle;
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::client::conn::http1::{handshake, SendRequest};
use hyper::{Request, Response as HyperResponse, StatusCode};
use hyper_util::rt::TokioIo;

// Async/Await
use std::io;
use tokio::spawn;
use tokio::task::JoinHandle;

// Error Handling
use log::{debug, info};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub struct Client;
impl Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

/// This struct is a convenience wrapper
/// around a russh client
pub struct SshConnection {
    pub uri: SshUri,
    pub handle: Option<StreamHandle>,
    pub ssh_handle: Option<SshHandle<Client>>,
}

impl Connection for SshConnection {
    async fn open(&mut self) -> Result<&mut Self, VirshleError> {
        self.open_with_agent().await?;
        self.connect_to_socket().await?;
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
}

impl SshConnection {
    /*
     * Open ssh connection to uri with keys from agent.
     */
    pub async fn open_with_agent(&mut self) -> Result<Self, VirshleError> {
        let uri = &self.uri;
        // Ssh connection vars
        let addrs = format!("{}:{}", uri.host, uri.port);
        let socket = uri.path.clone();
        let user = uri.user.clone();

        let mut agent = AgentClient::connect_env().await?;
        let agent_keys: Vec<PublicKey> = agent.request_identities().await?;

        let config = Config::default();
        let config = Arc::new(config);

        for key in agent_keys {
            // Fix: Initiate a new session for each agent keys
            // to circumvent server max_auth_tries.
            let sh = Client {};
            let mut handle = connect(config.clone(), addrs.clone(), sh).await?;

            let auth_res = handle
                .authenticate_publickey_with(user.clone(), key, None, &mut agent)
                .await?;
            if auth_res.success() {
                self.ssh_handle = Some(handle);
            }
        }
        let message = "Couldn't establish connection with host.";
        let help = "Add keys to ssh-agent";
        Err(LibError::new(message, help).into())
    }

    /*
     * After ssh connection is open.
     * Connect to socket at path: self.uri.path.
     */
    pub async fn connect_to_socket(&mut self) -> Result<(), VirshleError> {
        if let Some(ssh_handle) = &self.ssh_handle {
            let socket = &self.uri.path;
            println!("{}", socket);
            let channel = ssh_handle.channel_open_direct_streamlocal(socket).await;
            match channel {
                Ok(channel) => {
                    info!("Connected to socket at {:?}", self.uri);
                    let stream: TokioIo<ChannelStream<Msg>> = TokioIo::new(channel.into_stream());

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
                }
                Err(e) => {
                    let message = format!("Couldn't connect to virshle socket at:\n{:?}", socket);
                    let help = format!("Is virshle running on host {:?} ?", socket);
                    WrapError::builder()
                        .msg(&message)
                        .help(&help)
                        .origin(Error::from_err(e));
                }
            };
        }
        Ok(())
    }

    // pub async fn close(&self) -> Result<(), VirshleError> {
    //     self.handle
    //         .disconnect(
    //             Disconnect::ByApplication,
    //             "Disconnectied by virshle cli.",
    //             "English",
    //         )
    //         .await?;
    //     Ok(())
    // }
}

pub fn request_to_string<T>(req: &Request<T>) -> Result<String, VirshleError>
where
    T: Serialize,
{
    let mut string = "".to_owned();

    string.push_str(&format!(
        "{} {} {:?}\n",
        req.method(),
        req.uri(),
        req.version()
    ));

    for (key, value) in req.headers() {
        let key = key.to_string().to_case(Case::Title);
        let value = value.to_str().unwrap();
        string.push_str(&format!("{key}: {value}\n",));
    }

    let body: Value = serde_json::to_value(req.body().to_owned())?;
    match body {
        Value::Null => {}
        _ => {
            let body: String = serde_json::to_string(req.body().to_owned())?;
            string.push_str(&format!("\n{}\n", body));
        }
    };
    Ok(string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_to_localhost_ssh_server_and_socket() -> Result<()> {
        let uri = "ssh://deku";
        // let session = SshConnection::open_with_agent(uri).await?;
        // session.connect_to_socket().await?;
        // session.close().await?;
        Ok(())
    }

    // #[tokio::test]
    async fn send_request_to_socket() -> Result<()> {
        let request = Request::builder()
            .uri("/vm/list")
            .method("GET")
            .header("Host", "localhost")
            .body(())
            .into_diagnostic()?;

        let req = request_to_string(&request).into_diagnostic()?;
        println!("\n{}", req);

        let node = Node {
            name: "default".to_owned(),
            url: "ssh://deku".to_owned(),
        };

        // let node_connection = NodeConnection::from(&node);
        // let mut session = node_connection.open().await?;

        // session.put(&req).await?;

        Ok(())
    }
}
