// Http
use crate::http_cli::Connection;

// Error handling
use log::info;
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

// Hypervisor
use crate::config::VirshleConfig;

pub struct Client;

impl Client {
    // Get node url and connect
    async fn connection(&self) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;

        for node in config.get_nodes()? {
            println!("{:?}", node);
            // node.connect();
            // let socket = self.get_socket()?;
            // Connection::open(&socket).await
        }
        Ok(())
    }

    async fn get_all_vms() -> Result<(), VirshleError> {
        // let conn = Connection::open(&socket).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_nodes() -> Result<()> {
        Ok(())
    }
}
