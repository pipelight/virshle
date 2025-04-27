use std::collections::HashMap;

// Http
use crate::http_cli::{Connection, HttpRequest, NodeConnection};
use crate::{config::Node, Vm};

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

        let mut vms: Vec<Vm> = vec![];
        for node in config.get_nodes()? {
            let node_vms: Vec<Vm> = node.open().await?.get("/vm/list").await?.to_value().await?;
            vms.extend(node_vms);
            // node.connect();
            // let socket = self.get_socket()?;
            // Connection::open(&socket).await
        }
        Ok(())
    }

    /*
     * Display vms by node.
     */
    pub async fn display_all_vms() -> Result<(), VirshleError> {
        let e = Self::get_all_vms().await?;
        Vm::display_by_nodes(e).await?;
        Ok(())
    }

    /*
     * Get vms by node.
     */
    pub async fn get_all_vms() -> Result<HashMap<Node, Vec<Vm>>, VirshleError> {
        let config = VirshleConfig::get()?;

        let mut vms: HashMap<Node, Vec<Vm>> = HashMap::new();
        for node in config.get_nodes()? {
            let mut conn = node.open().await?;
            let node_vms: Vec<Vm> = conn.get("/vm/list").await?.to_value().await?;
            conn.close();
            vms.insert(node, node_vms);
        }
        Ok(vms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_all_vms() -> Result<()> {
        Client::get_all_vms().await?;
        Ok(())
    }
}
