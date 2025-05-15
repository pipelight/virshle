/*
* This module is to connect to a virshle instance through ssh.
*/

use crate::config::Node;
use crate::http_api::Server;

use super::{Connection, ConnectionHandle, ConnectionState, NodeConnection};
use super::{SshUri, Uri};

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
use log::{info, trace};
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
#[derive(Default)]
pub struct SshConnection {
    pub uri: SshUri,
    pub handle: Option<StreamHandle>,
    pub ssh_handle: Option<SshHandle<Client>>,
    pub state: ConnectionState,
}

impl ConnectionHandle for SshConnection {
    async fn open(&mut self) -> Result<&mut Self, VirshleError> {
        self.open_with_agent().await?;
        self.connect_to_socket().await?;
        Ok(self)
    }
    async fn close(&self) -> Result<(), VirshleError> {
        if let Some(ssh_handle) = &self.ssh_handle {
            ssh_handle
                .disconnect(
                    Disconnect::ByApplication,
                    "Disconnectied by virshle cli.",
                    "English",
                )
                .await?;
        }
        Ok(())
    }
}

impl SshConnection {
    pub fn new(url: &str) -> Result<Self, VirshleError> {
        let ssh_uri = SshUri::new(url)?;
        Ok(Self {
            uri: ssh_uri,
            ..Default::default()
        })
    }
}

impl SshConnection {
    /*
     * Open ssh connection to uri with keys from agent.
     */
    pub async fn open_with_agent(&mut self) -> Result<&mut Self, VirshleError> {
        let uri = &self.uri;
        // Ssh connection vars
        let addrs = format!("{}:{}", uri.host, uri.port);
        let socket = uri.path.clone();
        let user = uri.user.clone();

        let mut agent = AgentClient::connect_env().await?;
        let agent_keys: Vec<PublicKey> = agent.request_identities().await?;

        let config = Config::default();
        let config = Arc::new(config);

        // Put this block code in the "for" loop to:
        // Initiate a new session for each agent keys
        // to circumvent server max_auth_tries per tcp.
        // Or increase ssh server max_auth_tries
        let sh = Client {};
        let mut handle = connect(config.clone(), addrs.clone(), sh).await?;

        for key in agent_keys {
            let auth_res = handle
                .authenticate_publickey_with(user.clone(), key, None, &mut agent)
                .await?;

            if auth_res.success() {
                self.ssh_handle = Some(handle);
                return Ok(self);
            }
        }
        // If neither of the keys did work.
        let message = "Ssh authentication to host failed.";
        let help = "Add keys to ssh-agent";
        Err(LibError::new(message, help).into())
    }

    /*
     * After ssh connection is open.
     * Connect to socket at path: self.uri.path.
     */
    pub async fn connect_to_socket(&mut self) -> Result<&mut Self, VirshleError> {
        if let Some(ssh_handle) = &self.ssh_handle {
            let socket = &self.uri.path;
            let channel = ssh_handle.channel_open_direct_streamlocal(socket).await;
            match channel {
                Err(e) => {
                    let message = format!("Couldn't connect to socket: {}", socket);
                    let help = format!("Does the socket exist?");
                    let err = WrapError::builder()
                        .msg(&message)
                        .help(&help)
                        .origin(Error::from_err(e))
                        .build();
                    return Err(err.into());
                }
                Ok(channel) => {
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
                            info!("Connected to socket at {}", self.uri);
                            self.handle = Some(StreamHandle {
                                sender,
                                connection: spawn(async move { connection.await }),
                            });
                        }
                    };
                }
            };
        }
        Ok(self)
    }
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
        let node = Node {
            name: "test".to_owned(),
            url: "ssh://anon@deku".to_owned(),
        };
        let mut conn = NodeConnection::from(&node);
        conn.open().await?;
        conn.close().await?;
        Ok(())
    }
}
