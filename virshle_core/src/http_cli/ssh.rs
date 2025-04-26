/*
* This module is to connect to a virshle/libvirshle instance through ssh.
*/

use super::Connection;
use crate::config::{SshUri, Uri};
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

use russh::client::{connect, Config, Handle, Handler};
use russh::keys::agent::client::AgentClient;
use russh::{
    keys::load_secret_key,
    keys::{ssh_key::Algorithm, PrivateKeyWithHashAlg, PublicKey},
    ChannelMsg, CryptoVec, Disconnect,
};
use std::net::TcpStream;
use std::sync::Arc;
// Http
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
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

struct Client {}

// More SSH event handlers
// can be defined in this trait
// In this example, we're only using Channel, so these aren't needed.

// #[async_trait]
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
    uri: SshUri,
    session: Handle<Client>,
}

impl Connection for SshConnection {}

impl SshConnection {
    /*
     * Open ssh connection to uri with keys from agent.
     */
    pub async fn connect_with_agent(uri: &str) -> Result<Self, VirshleError> {
        let uri = Uri::new(uri)?;
        match uri.clone() {
            Uri::SshUri(uri) => {
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
                    let mut session = connect(config.clone(), addrs.clone(), sh).await?;

                    let auth_res = session
                        .authenticate_publickey_with(user.clone(), key, None, &mut agent)
                        .await?;
                    if auth_res.success() {
                        return Ok(Self { session, uri });
                    }
                }
            }
            _ => {
                let message = "Couldn't establish connection with host.";
                let help = "Bad uri provided";
                return Err(LibError::new(message, help).into());
            }
        };
        let message = "Couldn't establish connection with host.";
        let help = "Add keys to ssh-agent";
        Err(LibError::new(message, help).into())
    }

    /*
     * After ssh connection is open.
     * Connect to socket at path: self.uri.path.
     */
    pub async fn connect_to_socket(&self) -> Result<(), VirshleError> {
        let socket = &self.uri.path;
        println!("{}", socket);
        let channel = self.session.channel_open_direct_streamlocal(socket).await;
        match channel {
            Ok(channel) => {
                info!("Connected to socket at {:?}", self.uri);
                // let stream : TokioIo<UnixStream> =  tokio::io::
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
        // let ssh_channel = self
        //     .session
        //     .channel_open_direct_streamlocal("/var/lib/virshle/virshle.sock")
        //     .await?;

        // let mut ssh_stream = ssh_channel.into_stream();
        // tokio::io::copy_bidirectional(&mut local_socket, &mut ssh_stream)
        //     .await
        //     .expect("Copy error between local socket and SSH stream");

        Ok(())
    }

    pub async fn put(&mut self, req: &str) -> Result<(), VirshleError> {
        let mut channel = self.session.channel_open_session().await?;
        // channel.request_shell(true).await?;
        // let mut stream = channel.into_stream();
        // stream.write(b"notify-send ssh");

        // -U creates a binding to unixsocket.
        // -N closes the connection as soon as message sended.
        let cmd = format!("echo \"{}\" | nc -U /var/lib/virshle/virshle.socket", req);
        println!("\n{}", cmd);
        channel.exec(true, cmd).await?;

        let mut stdout = tokio::io::stdout();
        let mut code = None;

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
                    // Close channel as soon as response returned.
                    channel.close().await?;
                }
                // The command has returned an exit code
                ChannelMsg::ExitStatus { exit_status } => {
                    code = Some(exit_status);
                    // cannot leave the loop immediately,
                    // there might still be more data to receive
                }
                ChannelMsg::Eof | ChannelMsg::Close => {
                    break;
                }
                _ => {}
            };
        }

        // channel.close().await?;

        if let Some(exit_status) = code {
            if ExitStatus::from_raw(exit_status as i32).success() {
            } else {
                let message = "Couldn't call virshle socket successfully on remote host";
                let help = "Is virshle running on remote?";
                return Err(LibError::new(message, help).into());
            }
        } else {
            let message = "Command returned no exit code.";
            let help = "Connection might have been interupted.";
            return Err(LibError::new(message, help).into());
        }
        Ok(())
    }

    pub async fn close(&self) -> Result<(), VirshleError> {
        self.session
            .disconnect(
                Disconnect::ByApplication,
                "Disconnectied by virshle cli.",
                "English",
            )
            .await?;
        Ok(())
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

    // #[tokio::test]
    async fn connect_to_localhost_ssh_server() -> Result<()> {
        let uri = "ssh://deku";
        let session = SshConnection::connect_with_agent(uri).await?;
        session.disconnect().await?;
        Ok(())
    }
    #[tokio::test]
    async fn connect_to_localhost_ssh_server_and_socket() -> Result<()> {
        let uri = "ssh://deku";
        let session = SshConnection::connect_with_agent(uri).await?;
        session.connect_to_socket().await?;
        session.disconnect().await?;
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

        let uri = "ssh://deku";
        let mut session = SshConnection::connect_with_agent(uri).await?;

        session.put(&req).await?;

        Ok(())
    }
}
