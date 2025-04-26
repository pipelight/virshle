use crate::http_api::Server;
use crate::http_cli::Connection;
use std::fmt;
use url::Url;

use serde::{Deserialize, Serialize};
use users::{get_current_uid, get_user_by_uid};

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Node {
    pub name: String,
    pub url: String,
}
impl Default for Node {
    fn default() -> Self {
        let url = "unix://".to_owned() + &Server::get_socket().unwrap();
        Self {
            name: "default".to_owned(),
            url,
        }
    }
}

impl Node {
    pub fn connect(&self) -> Result<(), VirshleError> {
        match Uri::new(&self.url)? {
            Uri::SshUri(v) => {}
            Uri::LocalUri(v) => {
                Connection::open(&v.path);
            }
        };
        Ok(())
    }
}

/*
* Remote uris is greatly inspired by libvirt uri specs.
* driver[+transport]://[username@][hostname][:port]/[path][?extraparameters]
* https://libvirt.org/uri.html
*
* It only provides **local** and **ssh** connectors.
*
* example:
* - "file:///path/to/socket"
* - "ssh:///admin@server1/path/to/socket"
*/
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Uri {
    LocalUri(LocalUri),
    // A connection to
    SshUri(SshUri),
}
impl fmt::Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            Uri::SshUri(uri) => uri.to_string(),
            Uri::LocalUri(uri) => uri.to_string(),
        };
        write!(f, "{}", string)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SshUri {
    pub user: String,
    pub host: String,
    pub path: String,
    pub port: u64,
}
impl Default for SshUri {
    fn default() -> Self {
        let user = get_user_by_uid(get_current_uid()).unwrap();
        let username = user.name().to_str().unwrap().to_owned();
        Self {
            user: username,
            host: "localhost".to_owned(),
            path: Server::get_socket().unwrap(),
            port: 22,
        }
    }
}
impl fmt::Display for SshUri {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ssh://{}@{}:{}", self.user, self.host, self.path)
    }
}
impl fmt::Display for LocalUri {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unix://{}", self.path)
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalUri {
    path: String,
}

impl Uri {
    pub fn new(string: &str) -> Result<Self, VirshleError> {
        let url = Url::parse(string)?;
        match url.scheme() {
            "ssh" => Ok(Self::SshUri(Self::parse_ssh_url(&url)?)),
            "unix" => Ok(Self::LocalUri(Self::parse_local_url(&url)?)),
            _ => Err(
                LibError::new("Couldn't determine the uri scheme", "Try ssh:// or file://").into(),
            ),
        }
    }
    /*
     * Helper to easily parse a url with lacking segments into a virshle ssh uri.
     */
    fn parse_ssh_url(url: &Url) -> Result<SshUri, VirshleError> {
        let mut uri = SshUri::default();
        // Set host if some or fallback to default localhost.
        if let Some(host) = url.host_str() {
            uri.host = host.to_owned();
        }
        // Set username if some.
        if !url.username().is_empty() {
            uri.user = url.username().to_owned();
        }
        // An empty path is parsed as "/" by the Url lib.
        // Set path if a non empty one is set.
        if url.path() != "/" {
            uri.path = url.path().to_owned();
        }
        // Set port if some.
        if let Some(port) = url.port() {
            uri.port = port.into();
        }
        Ok(uri)
    }
    /*
     * Helper to easily parse a url with lacking segments into a virshle socket uri.
     */
    fn parse_local_url(url: &Url) -> Result<LocalUri, VirshleError> {
        let mut uri = LocalUri::default();
        // An empty path is parsed as "/" by the Url lib.
        if url.path() != "/" {
            uri.path = url.path().to_owned();
        }
        Ok(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn try_parse_default_uri() -> Result<()> {
        let uri = "unix:///path/to/socket";
        let url = Uri::LocalUri(LocalUri {
            path: "/path/to/socket".to_owned(),
        });
        let res = Uri::new(uri)?;
        assert_eq!(url, res);
        Ok(())
    }
    #[tokio::test]
    async fn try_parse_ssh_uri() -> Result<()> {
        let uri = "ssh://anon@server/path/to/socket";
        let url = Uri::SshUri(SshUri {
            user: "anon".to_owned(),
            host: "server".to_owned(),
            path: "/path/to/socket".to_owned(),
            port: 22,
        });
        let res = Uri::new(uri)?;
        assert_eq!(url, res);
        Ok(())
    }
}
