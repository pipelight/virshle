/*
* This module is to connect to a virshle/libvirshle instance through ssh.
*/

use super::uri::{LibvirtUri, SshUri};
use ssh2::Session;
use std::net::TcpStream;

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

pub struct SshConnection {
    sender: SendRequest<Full<Bytes>>,
    connection: JoinHandle<Result<(), hyper::Error>>,
}

pub struct SshUnixStream {}

impl SshConnection {
    pub fn connect(uri: SshUri) -> io::Result<SshUnixStream> {
        // Connect to the local SSH server
        let tcp = TcpStream::connect("127.0.0.1:22").unwrap();

        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        // Try to authenticate with the first identity in the agent.
        sess.userauth_agent("username").unwrap();

        // Make sure we succeeded
        assert!(sess.authenticated());
    }
}

// impl<'a> std::io::Write for &'a SshConnection {
//     fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
//         self.0.write(buf)
//     }
//     fn flush(&mut self) -> std::io::Result<()> {
//         Ok(())
//     }
// }
// impl std::io::Write for SshConnection {
//     fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
//         std::io::Write::write(&mut &*self, buf)
//     }
//     fn flush(&mut self) -> std::io::Result<()> {
//         std::io::Write::flush(&mut &*self)
//     }
// }

// impl SshConnection {
//     pub async fn open(socket: &str) -> Result<Self, VirshleError> {
//         // Connect to the local SSH server
//         let tcp = TcpStream::connect("127.0.0.1:22").unwrap();
//         let mut sess = Session::new().unwrap();
//         sess.set_tcp_stream(tcp);
//         sess.handshake().unwrap();
//
//         // Try to authenticate with the first identity in the agent.
//         sess.userauth_agent("username").unwrap();
//
//         // Make sure we succeeded
//         assert!(sess.authenticated());
//     }
// }
