/*
* Cloud hypervisor compatibility layer.
*
* This crate is an api to easily connect to multiple endpoints and send/receive data streams.
*
* - Local unix sockets
* - Tcp
* - Unix sockets behind ssh.
*
* It is combined with the HttpRequest trait to send/receive http between enpoints
* with trivial methods like .get(), put(json(data)).
*
* Sources:
* https://levelup.gitconnected.com/learning-rust-http-via-unix-socket-fee3241b4340
* https://github.com/amacal/etl0/blob/85d155b1cdf2f7962188cd8b8833442a1e6a1132/src/etl0/src/docker/http.rs
* https://docs.rs/hyperlocal/latest/hyperlocal/
*/

mod uri;

// Main connection types.
mod socket;
mod ssh;
mod tcp;

// Reexport
pub use socket::UnixConnection;
pub use ssh::SshConnection;
pub use tcp::TcpConnection;
pub use uri::{LocalUri, SshUri, TcpUri, Uri};

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

// Stream
use russh::{client::Msg, ChannelStream};
use tokio::net::{TcpStream, UnixStream};

use std::convert::Into;
use std::future::Future;

// Error Handling
use miette::Result;
use virshle_error::VirshleError;

/*
* An unused trait that should have enabled usage of multiple stream types (not working).
* For now, usage of known types in enumeration is preffered.
*/
pub trait Streamable:
// tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + Sized
// tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + Sync
tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send {}
// pub trait Streamable: hyper::rt::Read + hyper::rt::Write {}
// pub trait Streamable: std::io::Read + std::io::Write {}

/// An enumeration of allowed stream types.
pub enum Stream {
    Ssh(ChannelStream<Msg>),
    Socket(UnixStream),
    Tcp(TcpStream),
}
impl Streamable for ChannelStream<Msg> {}
impl Streamable for UnixStream {}

pub trait ConnectionHandle {
    // fn open(&mut self) -> impl Future<Output = Result<&mut Self, VirshleError>> + Send;
    fn open(&mut self) -> impl Future<Output = Result<Stream, VirshleError>> + Send;
    fn close(&mut self) -> impl Future<Output = Result<(), VirshleError>> + Send;
    fn get_state(&mut self) -> impl Future<Output = Result<ConnectionState, VirshleError>> + Send;
}

pub enum Connection {
    SshConnection(SshConnection),
    UnixConnection(UnixConnection),
    TcpConnection(TcpConnection),
}

impl ConnectionHandle for Connection {
    async fn open(&mut self) -> Result<Stream, VirshleError> {
        match self {
            Connection::SshConnection(e) => e.open().await,
            Connection::UnixConnection(e) => e.open().await,
            Connection::TcpConnection(e) => e.open().await,
        }
    }
    async fn close(&mut self) -> Result<(), VirshleError> {
        match self {
            Connection::SshConnection(e) => {
                e.close().await?;
            }
            Connection::UnixConnection(e) => {
                e.close().await?;
            }
            Connection::TcpConnection(e) => {
                e.close().await?;
            }
        };
        Ok(())
    }
    async fn get_state(&mut self) -> Result<ConnectionState, VirshleError> {
        match self {
            Connection::SshConnection(e) => e.get_state().await,
            Connection::UnixConnection(e) => e.get_state().await,
            Connection::TcpConnection(e) => e.get_state().await,
        }
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub enum ConnectionState {
    /// Success: Connection established and daemon is up!
    DaemonUp,

    /// Uninitialized: Connection not established.
    #[default]
    Down,
    // Warning: Small error
    SshAuthError,
    // Error
    DaemonDown,
    SocketNotFound,
    /// Unknown network reason.
    Unreachable,
}
impl ConnectionState {
    pub fn display(&self) -> String {
        let icon = "●";
        let res = match self {
            // Success
            ConnectionState::DaemonUp => format!("{} Running", icon).green().to_string(),
            // Uninitialized
            ConnectionState::Down => format!("{} Down", icon).white().to_string(),
            // Warning: small error
            ConnectionState::SshAuthError => format!("{} SshAuthError", icon).yellow().to_string(),
            // Error
            ConnectionState::SocketNotFound => format!("{} SocketNotFound", icon).red().to_string(),
            ConnectionState::DaemonDown => format!("{} DaemonDown", icon).red().to_string(),
            // Unknown network reason.
            ConnectionState::Unreachable => format!("{} Unreachable", icon).red().to_string(),
        };
        format!("{}", res)
    }
}
