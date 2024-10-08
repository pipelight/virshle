/*
* This module is to connect to a virshle/libvirshle instance through ssh.
*/

use super::uri::{LibvirtUri, SshUri};
use std::net::TcpStream;

use russh::client::{connect, Config, Handle, Handler};
use russh::keys::agent::client::AgentClient;
use russh::keys::{key::PublicKey, load_secret_key};
use russh::{ChannelMsg, CryptoVec, Disconnect};

use std::pin::Pin;
use std::sync::Arc;

use std::path::Path;
use std::time::Duration;

// Ssh
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeekExt, AsyncWrite, AsyncWriteExt};
use tokio::net::ToSocketAddrs;
use tokio::net::UnixStream;

// Http cli
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::client::conn::http1::{handshake, SendRequest};
use hyper::{Request, Response as HyperResponse, StatusCode};
use hyper_util::rt::TokioIo;

// Async/Await
use std::io;
use tokio::task::JoinHandle;

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

struct Client {}

// More SSH event handlers
// can be defined in this trait
// In this example, we're only using Channel, so these aren't needed.
#[async_trait]
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
pub struct Session {
    session: Handle<Client>,
}

impl Session {
    async fn connect<P: AsRef<Path>, A: ToSocketAddrs>(
        key_path: P,
        user: impl Into<String>,
        addrs: A,
    ) -> Result<Self, VirshleError> {
        let key_pair = load_secret_key(key_path, None)?;
        let config = Config {
            inactivity_timeout: Some(Duration::from_secs(5)),
            ..<_>::default()
        };

        let config = Arc::new(config);
        let sh = Client {};

        let mut session = connect(config, addrs, sh).await?;
        let auth_res = session
            .authenticate_publickey(user, Arc::new(key_pair))
            .await?;

        if !auth_res {
            // anyhow::bail!("Authentication failed");
        }

        Ok(Self { session })
    }

    async fn call(&mut self, command: &str) -> Result<u32, VirshleError> {
        let mut channel = self.session.channel_open_session().await?;
        channel.exec(true, command).await?;

        let mut code = None;
        let mut stdout = tokio::io::stdout();

        loop {
            // There's an event available on the session channel
            let Some(msg) = channel.wait().await else {
                break;
            };
            match msg {
                // Write data to the terminal
                ChannelMsg::Data { ref data } => {
                    stdout.write_all(data).await?;
                    stdout.flush().await?;
                }
                // The command has returned an exit code
                ChannelMsg::ExitStatus { exit_status } => {
                    code = Some(exit_status);
                    // cannot leave the loop immediately, there might still be more data to receive
                }
                _ => {}
            }
        }
        Ok(code.expect("program did not exit cleanly"))
    }

    async fn close(&mut self) -> Result<(), VirshleError> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}

pub struct SshConnection {}

impl SshConnection {
    pub async fn open() -> Result<(), VirshleError> {
        let mut agent = AgentClient::connect_env().await?;

        // let public_key;
        let mut public_keys: Vec<PublicKey> = vec![];
        for key in agent.request_identities().await? {
            public_keys.push(key);
        }

        let user = "anon";
        let addrs = "127.0.0.1:22";

        let config = Config {
            inactivity_timeout: Some(Duration::from_secs(10)),
            ..<_>::default()
        };

        let config = Arc::new(config);
        let sh = Client {};

        let mut session = connect(config, addrs, sh).await?;

        for key in public_keys {
            let agent = AgentClient::connect_env().await?;
            let (_, auth_res) = session.authenticate_future(user, key, agent).await;

            if auth_res? {
                let mut channel = session.channel_open_session().await?;

                // channel.request_shell(true).await?;
                // let mut stream = channel.into_stream();
                // stream.write(b"notify-send ssh");

                channel.exec(true, "notify-send ssh").await?;
                let mut code = None;
                let mut stdout = tokio::io::stdout();

                loop {
                    // There's an event available on the session channel
                    let Some(msg) = channel.wait().await else {
                        break;
                    };
                    match msg {
                        // Write data to the terminal
                        ChannelMsg::Data { ref data } => {
                            stdout.write_all(data).await?;
                            stdout.flush().await?;
                        }
                        // The command has returned an exit code
                        ChannelMsg::ExitStatus { exit_status } => {
                            code = Some(exit_status);
                            // cannot leave the loop immediately, there might still be more data to receive
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_to_ssh() -> Result<()> {
        SshConnection::open().await?;
        Ok(())
    }
}
