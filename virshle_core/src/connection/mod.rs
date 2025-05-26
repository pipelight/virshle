/*
* Cloud hypervisor compatibility layer.
*
* This crate is an api to easily connect to multiple endpoints and send/receive data streams.
*
* - Local unix sockets
* - unix sockets behind ssh.
*
* It is combined with the HttpRequest trait to send/receive http between enpoints
* with trivial methods like .get(), put(json(data)).
*
* Sources:
* https://levelup.gitconnected.com/learning-rust-http-via-unix-socket-fee3241b4340
* https://github.com/amacal/etl0/blob/85d155b1cdf2f7962188cd8b8833442a1e6a1132/src/etl0/src/docker/http.rs
* https://docs.rs/hyperlocal/latest/hyperlocal/
*/

// Main connection types.

mod socket;
mod ssh;
mod state;
mod uri;

// Reexport
pub use socket::UnixConnection;
pub use ssh::SshConnection;
pub use state::ConnectionState;
pub use uri::{LocalUri, SshUri, Uri};

use crate::cloud_hypervisor::Vm;
use crate::config::Node;

// Http
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::client::conn::http1; // {handshake, SendRequest};
use hyper::client::conn::http2; // {handshake};
use hyper::Request;

// Async/Await
use tokio::task::JoinHandle;

// Stream
use russh::{client::Msg, ChannelStream};
use tokio::net::UnixStream;

use serde::{Deserialize, Serialize};
use std::future::Future;

// Error Handling
use miette::{Error, Result};
use virshle_error::{ConnectionError, VirshleError, WrapError};

/*
* An unused trait that should have enabled usage of multiple stream types (not working).
* For now, usage of known types in enumeration is preffered.
*/
pub trait Streamable:
    // tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + Sized
// tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send + Sync
tokio::io::AsyncRead + tokio::io::AsyncWrite + std::marker::Unpin + Send
{
}
// pub trait Streamable: hyper::rt::Read + hyper::rt::Write {}
// pub trait Streamable: std::io::Read + std::io::Write {}

/*
* An enumeration of allowed stream types.
*/
pub enum Stream {
    Ssh(ChannelStream<Msg>),
    Socket(UnixStream),
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
}

impl ConnectionHandle for Connection {
    async fn open(&mut self) -> Result<Stream, VirshleError> {
        match self {
            Connection::SshConnection(e) => e.open().await,
            Connection::UnixConnection(e) => e.open().await,
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
        };
        Ok(())
    }
    async fn get_state(&mut self) -> Result<ConnectionState, VirshleError> {
        match self {
            Connection::SshConnection(e) => e.get_state().await,
            Connection::UnixConnection(e) => e.get_state().await,
        }
    }
}

impl From<&Node> for Connection {
    fn from(value: &Node) -> Self {
        match Uri::new(&value.url).unwrap() {
            Uri::SshUri(v) => Connection::SshConnection(SshConnection {
                uri: v,
                ssh_handle: None,
            }),
            Uri::LocalUri(v) => Connection::UnixConnection(UnixConnection { uri: v }),
        }
    }
}

impl From<&Vm> for Connection {
    fn from(value: &Vm) -> Self {
        let uri = value.get_socket_uri().unwrap();
        match Uri::new(&uri).unwrap() {
            Uri::SshUri(v) => Connection::SshConnection(SshConnection {
                uri: v,
                ssh_handle: None,
            }),
            Uri::LocalUri(v) => Connection::UnixConnection(UnixConnection { uri: v }),
        }
    }
}
impl From<&mut Vm> for Connection {
    fn from(value: &mut Vm) -> Self {
        let uri = value.get_socket().unwrap();
        match Uri::new(&uri).unwrap() {
            Uri::SshUri(v) => Connection::SshConnection(SshConnection {
                uri: v,
                ssh_handle: None,
            }),
            Uri::LocalUri(v) => Connection::UnixConnection(UnixConnection { uri: v }),
        }
    }
}
