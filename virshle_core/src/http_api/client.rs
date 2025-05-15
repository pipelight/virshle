use std::collections::HashMap;

// Connections and Http
use crate::connection::{Connection, ConnectionHandle, NodeConnection};
use crate::http_request::{HttpRequest, HttpSender};

use crate::cli::VmArgs;
use crate::{Node, NodeInfo, Vm, VmState, VmTemplate};
use std::str::FromStr;

// Error handling
use log::{error, warn};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

// Hypervisor
use crate::config::VirshleConfig;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
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
    pub async fn get_all_templates() -> Result<HashMap<Node, Vec<VmTemplate>>, VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut templates: HashMap<Node, Vec<VmTemplate>> = HashMap::new();
        for node in nodes {
            match node.open().await {
                Err(e) => {
                    warn!("{}", e);
                }
                Ok(mut conn) => {
                    let node_templates: Vec<VmTemplate> =
                        conn.get("/template/list").await?.to_value().await?;
                    conn.close();
                    templates.insert(node, node_templates);
                }
            }
        }
        Ok(templates)
    }

    pub async fn get_nodes_info() -> Result<HashMap<Node, Option<NodeInfo>>, VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut node_info: HashMap<Node, Option<NodeInfo>> = HashMap::new();
        for node in nodes {
            match node.open().await {
                Err(e) => {
                    error!("{}", e);
                    node_info.insert(node, None);
                }
                Ok(mut conn) => {
                    let res: NodeInfo = conn.get("/node/info").await?.to_value().await?;
                    conn.close();
                    node_info.insert(node, Some(res));
                }
            }
        }
        Ok(node_info)
    }
    /*
     * Get a hashmap/dict of all vms per (reachable) node.
     */
    pub async fn get_all_vm() -> Result<HashMap<Node, Vec<Vm>>, VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut vms: HashMap<Node, Vec<Vm>> = HashMap::new();
        for node in nodes {
            match node.open().await {
                Err(e) => {
                    error!("{}", e);
                }
                Ok(mut conn) => {
                    let node_vms: Vec<Vm> = conn.get("/vm/list").await?.to_value().await?;
                    conn.close();
                    vms.insert(node, node_vms);
                }
            }
        }
        Ok(vms)
    }
    /*
     * Filter vms based on args.
     */
    pub async fn filter(
        mut nodes: HashMap<Node, Vec<Vm>>,
        args: VmArgs,
    ) -> Result<HashMap<Node, Vec<Vm>>, VirshleError> {
        let config = VirshleConfig::get()?;

        // Filter Nodes by name
        if let Some(node_name) = &args.node {
            nodes.iter_mut().filter(|(k, _)| &k.name == node_name);
        }

        // Filter Vms by State
        if let Some(state) = &args.state {
            for (node, vms) in &mut nodes {
                let state = VmState::from_str(state).unwrap();
                for (i, vm) in vms.clone().iter().enumerate() {
                    if vm.get_state().await? != state {
                        vms.remove(i);
                    }
                }
            }
        }
        Ok(nodes)
    }
    /* */
    pub async fn start_vm(args: VmArgs) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut vms: HashMap<Node, Vec<Vm>> = HashMap::new();
        for node in nodes {}
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_all_vm() -> Result<()> {
        Client::get_all_vm().await?;
        Ok(())
    }
    #[tokio::test]
    async fn test_get_all_vm_w_filter() -> Result<()> {
        let args = VmArgs {
            node: Some("default".to_owned()),
            ..Default::default()
        };
        // Client::get_all_vm_and_filter(args).await?;
        Ok(())
    }
}
