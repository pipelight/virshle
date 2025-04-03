/*
* This module is to connect to a virshle/libvirshle instance through ssh.
*/

use crate::config::uri::{SshUri, Uri};
use std::net::TcpStream;

use russh::client::{connect, Config, Handle, Handler};
use russh::keys::agent::client::AgentClient;
use russh::{
    keys::load_secret_key,
    keys::{PrivateKeyWithHashAlg, PublicKey},
    ChannelMsg, CryptoVec, Disconnect,
};
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;

use std::pin::Pin;
use std::sync::Arc;

use std::path::Path;
use std::time::Duration;

use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, Value};

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
// #[async_trait]/
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
            .authenticate_publickey(user, PrivateKeyWithHashAlg::new(Arc::new(key_pair), None))
            .await?;

        Ok(Self { session })
    }
}

pub struct SshConnection {}

impl Session {
    pub async fn open() -> Result<Self, VirshleError> {
        let mut agent = AgentClient::connect_env().await?;

        let mut public_keys: Vec<PublicKey> = vec![];
        for key in agent.request_identities().await? {
            public_keys.push(key);
        }

        let user = "anon";
        let addrs = "127.0.0.1:22";

        let config = Config {
            inactivity_timeout: Some(Duration::from_secs(5)),
            ..<_>::default()
        };

        let config = Arc::new(config);
        let sh = Client {};

        let mut session = connect(config, addrs, sh).await?;

        for key in public_keys {
            let mut agent = AgentClient::connect_env().await?;
            let auth_res = session
                .authenticate_publickey_with(user, key, None, &mut agent)
                .await?;
            if auth_res.success() {
                return Ok(Self { session });
            }
        }
        let message = "Couldn't establish connection with host.";
        let help = "Add keys to ssh-agent";
        Err(LibError::new(message, help).into())
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

    async fn close(&mut self) -> Result<(), VirshleError> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
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

    #[tokio::test]
    async fn connect_to_ssh() -> Result<()> {
        Session::open().await?;
        Ok(())
    }

    #[tokio::test]
    async fn send_request_to_socket() -> Result<()> {
        let request = Request::builder()
            .uri("/")
            .method("GET")
            .header("Host", "localhost")
            .body(())
            .into_diagnostic()?;

        let req = request_to_string(&request).into_diagnostic()?;
        println!("\n{}", req);

        let mut session = Session::open().await?;
        session.put(&req).await?;

        Ok(())
    }
}
