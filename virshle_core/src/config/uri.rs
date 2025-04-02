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

use url::Url;

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Uri {
    LocalUri(LocalUri),
    SshUri(SshUri),
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct SshUri {
    username: String,
    host: String,
    path: String,
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct LocalUri {
    path: String,
}

impl Uri {
    pub fn new(string: &str) -> Result<Self, VirshleError> {
        let url = Url::parse(string)?;
        match url.scheme() {
            "ssh" => Ok(Self::SshUri(SshUri {
                username: url.username().to_owned(),
                host: url.host().unwrap().to_string(),
                path: url.path().to_owned(),
            })),
            "file" => Ok(Self::LocalUri(LocalUri {
                path: url.path().to_owned(),
            })),
            _ => Err(
                LibError::new("Couldn't determine the uri scheme", "Try ssh:// or file://").into(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn try_parse_default_uri() -> Result<()> {
        let uri = "file:///path/to/socket";
        let url = Uri::LocalUri(LocalUri {
            path: "/path/to/socket".to_owned(),
        });
        let res = Uri::new(uri)?;
        assert_eq!(url, res);
        Ok(())
    }
    #[tokio::test]
    async fn try_parse_ssh_uri() -> Result<()> {
        let uri = "ssh://admin@server/path/to/socket";
        let url = Uri::SshUri(SshUri {
            username: "admin".to_owned(),
            host: "server".to_owned(),
            path: "/path/to/socket".to_owned(),
        });
        let res = Uri::new(uri)?;
        assert_eq!(url, res);
        Ok(())
    }
}
