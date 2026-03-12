use crate::peer::Peer;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

// Ssh
use rand_core::OsRng;
// use rand::rngs::OsRng;
use russh::keys::{ssh_key::Algorithm, PrivateKey, PublicKey};

// Error Handling
use miette::Result;
use tracing::{debug, info, trace};
use virshle_error::{LibError, VirshleError};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct NodeConfig {
    pub alias: Option<String>,
    pub private_key: String,
    pub public_key: String,
    pub passive: Option<bool>,
}
impl TryInto<Node> for NodeConfig {
    type Error = VirshleError;
    fn try_into(self) -> Result<Node, Self::Error> {
        (&self).try_into()
    }
}
impl TryInto<Node> for &NodeConfig {
    type Error = VirshleError;
    #[tracing::instrument]
    fn try_into(self) -> Result<Node, Self::Error> {
        #[cfg(not(debug_assertions))]
        let mut path = PathBuf::from("");
        #[cfg(debug_assertions)]
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(&self.private_key);
        path = path.as_path().canonicalize()?;
        trace!("Reading Self private_key at: {:#?}.", path);
        let private_key = fs::read_to_string(path)?;

        #[cfg(not(debug_assertions))]
        let mut path = PathBuf::from("");
        #[cfg(debug_assertions)]
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(&self.public_key);
        path = path.as_path().canonicalize()?;
        trace!("Reading Self public_key at: {:#?}.", path);
        let public_key = fs::read_to_string(path)?;
        Ok(Node {
            alias: Some("Self".to_owned()),
            private_key,
            public_key,
            passive: false,
        })
    }
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Node {
    pub alias: Option<String>,
    pub private_key: String,
    pub public_key: String,
    pub passive: bool,
}
impl Default for Node {
    fn default() -> Self {
        let key_pair = PrivateKey::random(&mut OsRng, Algorithm::Ed25519).unwrap();
        let public_key = key_pair.public_key().to_openssh().unwrap();
        let private_key = key_pair
            .to_openssh(russh::keys::ssh_key::LineEnding::LF)
            .unwrap()
            .to_string();
        Node {
            alias: Some("Self".to_owned()),
            private_key,
            public_key,
            passive: false,
        }
    }
}

impl Into<Peer> for Node {
    fn into(self) -> Peer {
        (&self).into()
    }
}
impl Into<Peer> for &Node {
    fn into(self) -> Peer {
        let url = "unix:///var/lib/virshle/virshle.sock".to_owned();
        Peer {
            alias: self.alias.clone(),
            url,
            weight: None,
            public_key: Some(self.public_key.clone()),
        }
    }
}

impl Node {
    pub fn did(&self) -> Result<String, VirshleError> {
        let pem = &self.public_key;
        let russh_key = russh::keys::PublicKey::from_str(pem).unwrap();
        let bytes: &[u8; 32] = russh_key.key_data().ed25519().unwrap().as_ref();
        let rad_key = radicle_crypto::PublicKey::from(*bytes);
        let did = rad_key.to_human();
        Ok(did)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::utils::testing;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[tokio::test]
    async fn load_testing_node_random_keys() -> Result<()> {
        testing::tracer()
            .verbosity(tracing::Level::TRACE)
            .db(true)
            .set()?;

        let node = Node::default();
        trace!("{:#?}", node);
        Ok(())
    }
    #[tokio::test]
    async fn testing_node_to_did() -> Result<()> {
        testing::tracer()
            .verbosity(tracing::Level::TRACE)
            .db(true)
            .set()?;

        let node = Node::default();
        let peer: Peer = node.into();
        let did = peer.did()?;
        trace!("Node did is: {:#?}", did);
        Ok(())
    }

    #[tokio::test]
    async fn load_testing_node_keys_from_file() -> Result<()> {
        testing::tracer()
            .verbosity(tracing::Level::TRACE)
            .db(true)
            .set()?;

        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let mut private_key = path.clone();
        private_key.push("./keys/default");
        let private_key = private_key.to_str().unwrap().to_owned();

        let mut public_key = path.clone();
        public_key.push("./keys/default.pub");
        let public_key = public_key.to_str().unwrap().to_owned();

        let config = NodeConfig {
            alias: None,
            private_key,
            public_key,
            passive: None,
        };
        let node: Node = config.try_into()?;
        trace!("{:#?}", node);

        Ok(())
    }

    #[tokio::test]
    async fn load_testing_node_keys_from_config_file() -> Result<()> {
        testing::tracer()
            .verbosity(tracing::Level::TRACE)
            .db(true)
            .set()?;

        let node = Config::get()?.node()?;
        trace!("{:#?}", node);
        Ok(())
    }
}
