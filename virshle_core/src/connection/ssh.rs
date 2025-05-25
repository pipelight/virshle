/*
* This module is to connect to a virshle instance through ssh.
*/

use crate::config::Node;

use super::{Connection, ConnectionHandle, ConnectionState, Stream};
use super::{SshUri, Uri};

use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use serde::{Deserialize, Serialize};

// Stream
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWrite, AsyncWriteExt};
use tokio::net::ToSocketAddrs;
use tokio::net::UnixStream;

// Ssh
use russh::client::{connect, Config, Handle as SshHandle, Msg};
use russh::keys::agent::client::AgentClient;
use russh::{
    keys::load_secret_key,
    keys::{ssh_key::Algorithm, PrivateKeyWithHashAlg, PublicKey},
    ChannelMsg, ChannelStream, CryptoVec, Disconnect,
};
use std::sync::Arc;

// Http
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::client::conn::http1; // {handshake};
use hyper::client::conn::http2; // {handshake};
use hyper::{Request, Response as HyperResponse, StatusCode};
use hyper_util::rt::TokioIo;

// Async/Await
use std::io;
use tokio::spawn;
use tokio::task::JoinHandle;

// Error Handling
use log::{info, trace};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{ConnectionError, LibError, VirshleError, WrapError};

pub struct SshClient;
impl russh::client::Handler for SshClient {
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
    pub ssh_handle: Option<SshHandle<SshClient>>,
}

impl ConnectionHandle for SshConnection {
    async fn open(&mut self) -> Result<Stream, VirshleError> {
        // Connect to ssh remote with agent
        if self.ssh_handle.is_none() {
            self.open_with_agent().await?;
        }
        let stream = self.connect_to_socket().await?;
        Ok(Stream::Ssh(stream))
    }
    async fn close(&mut self) -> Result<(), VirshleError> {
        if let Some(ssh_handle) = &self.ssh_handle {
            ssh_handle
                .disconnect(
                    Disconnect::ByApplication,
                    "Disconnectied by virshle cli.",
                    "English",
                )
                .await
                .map_err(|e| ConnectionError::from(e))?;
        }
        Ok(())
    }
    async fn get_state(&mut self) -> Result<ConnectionState, VirshleError> {
        let res = self.open().await;
        match res {
            Err(err) => match &err {
                VirshleError::ConnectionError(err) => match err {
                    ConnectionError::DaemonDown => Ok(ConnectionState::DaemonDown),
                    ConnectionError::SocketNotFound => Ok(ConnectionState::SocketNotFound),
                    ConnectionError::SshAuthError
                    | ConnectionError::RusshError(_)
                    | ConnectionError::SshKeyError(_)
                    | ConnectionError::SshAgentError(_) => Ok(ConnectionState::SshAuthError),
                },
                _ => Ok(ConnectionState::Unreachable),
            },
            Ok(conn) => Ok(ConnectionState::DaemonUp),
        }
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
    pub async fn open_with_agent(&mut self) -> Result<&mut Self, ConnectionError> {
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
        let sh = SshClient {};
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
        Err(ConnectionError::SshAuthError)
    }

    /*
     * After ssh connection is open.
     * Connect to socket at path: self.uri.path.
     */
    pub async fn connect_to_socket(&self) -> Result<ChannelStream<Msg>, VirshleError> {
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
                    let stream: ChannelStream<Msg> = channel.into_stream();
                    Ok(stream)
                }
            }
        } else {
            let message = format!("Ssh connection not opened.");
            let help = format!("First initiate the ssh connection.");
            let err = LibError::builder().msg(&message).help(&help).build();
            return Err(err.into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_to_localhost_ssh_server_and_socket() -> Result<()> {
        let node = Node::new("test", "ssh://anon@deku")?;
        let mut conn = Connection::from(&node);
        conn.open().await?;
        conn.close().await?;
        Ok(())
    }
}
