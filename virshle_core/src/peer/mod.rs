mod best;
mod display;
mod info;

use crate::config::{Config, VmTemplate};
pub use info::{HostCpu, HostDisk, HostInfo, HostRam, NodeInfo};
use std::str::FromStr;

// Connection
use virshle_network::connection::{
    Connection, ConnectionHandle, SshConnection, TcpConnection, UnixConnection, Uri,
};

use serde::{Deserialize, Serialize};

// Error Handling
use miette::{Error, Result};
use tracing::{trace, warn};
use virshle_error::{LibError, VirshleError, WrapError};

///A declaration of a remote/local virshle daemon virshle nodes (alias and address)
/// to be queried by the cli.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Peer {
    pub alias: String,
    pub url: String,
    pub weight: Option<i32>,
    pub public_key: Option<String>,
}
impl Peer {
    /// Convert peer public key into relatively human readable string.
    /// See radicle/heartwood crates for indepth functionning.
    pub fn did(&self) -> Result<String, VirshleError> {
        let mut did: String = "".to_owned();
        if let Some(pem) = &self.public_key {
            let public_key = russh::keys::PublicKey::from_str(pem).unwrap();
            let bytes: &[u8; 32] = public_key.key_data().ed25519().unwrap().as_ref();
            let rad_key = radicle_crypto::PublicKey::from(*bytes);
            did = rad_key.to_human();
        }
        Ok(did)
    }
}

impl Default for Peer {
    fn default() -> Self {
        // let url = "unix://".to_owned() + &NodeServer::get_socket().unwrap();
        let url = "unix:///var/lib/virshle/virshle.sock".to_owned();
        Self {
            alias: "default".to_owned(),
            url,
            weight: None,
            public_key: None,
        }
    }
}
impl Peer {
    pub fn new(alias: &str, url: &str) -> Result<Self, VirshleError> {
        let e = Peer {
            alias: alias.to_owned(),
            url: url.to_owned(),
            weight: None,
            public_key: None,
        };
        Ok(e)
    }
}

impl TryInto<Connection> for Peer {
    type Error = VirshleError;
    fn try_into(self) -> Result<Connection, Self::Error> {
        (&self).try_into()
    }
}
impl TryInto<Connection> for &Peer {
    type Error = VirshleError;
    fn try_into(self) -> Result<Connection, Self::Error> {
        let conn = match Uri::new(&self.url)? {
            Uri::SshUri(v) => Connection::SshConnection(SshConnection {
                uri: v,
                ssh_handle: None,
            }),
            Uri::LocalUri(v) => Connection::UnixConnection(UnixConnection { uri: v }),
            Uri::TcpUri(v) => Connection::TcpConnection(TcpConnection { uri: v }),
        };
        Ok(conn)
    }
}

impl Peer {
    /*
     * Open connection to node and return handler.
     */
    pub async fn open(&self) -> Result<Connection, VirshleError> {
        let mut conn: Connection = self.try_into()?;
        match conn.open().await {
            Ok(v) => return Ok(conn),
            Err(e) => {
                warn!("{}", e);
                let message = format!(
                    "Couldn't connect to virshle daemon on node {:?}.",
                    self.alias
                );
                let help = format!("Is virshle daemon running at url: {:?} ?", self.url);
                let err = WrapError::builder()
                    .msg(&message)
                    .help(&help)
                    .origin(Error::from_err(e))
                    .build();
                return Err(err.into());
            }
        };
    }
}
