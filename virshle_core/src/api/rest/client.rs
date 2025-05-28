use std::collections::HashMap;

// Connections and Http
use crate::connection::{Connection, ConnectionHandle, ConnectionState};
use crate::http_request::{Rest, RestClient};

use crate::cli::{StartArgs, VmArgs};
use crate::{Node, NodeInfo, Vm, VmState, VmTemplate};
use std::str::FromStr;

// Error handling
use log::{error, info, warn};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

// Hypervisor
use crate::config::VirshleConfig;

pub struct NodeRestClient;

impl NodeRestClient {
    pub async fn get_all_templates() -> Result<HashMap<Node, Vec<VmTemplate>>, VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut templates: HashMap<Node, Vec<VmTemplate>> = HashMap::new();
        for node in nodes {
            let mut conn = Connection::from(&node);
            match node.open().await {
                Err(e) => {
                    warn!("{}", e);
                }
                Ok(_) => {
                    let mut rest = RestClient::from(&mut conn);
                    let node_templates: Vec<VmTemplate> =
                        rest.get("/template/list").await?.to_value().await?;
                    templates.insert(node, node_templates);
                }
            }
        }
        Ok(templates)
    }

    pub async fn get_nodes_info(
    ) -> Result<HashMap<Node, (ConnectionState, Option<NodeInfo>)>, VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut node_info: HashMap<Node, (ConnectionState, Option<NodeInfo>)> = HashMap::new();
        for node in nodes {
            let mut conn = Connection::from(&node);
            match conn.open().await {
                Err(e) => {
                    error!("{}", e);
                    node_info.insert(node, (conn.get_state().await?, None));
                }
                Ok(_) => {
                    let state = conn.get_state().await?;
                    let mut rest = RestClient::from(&mut conn);
                    let res: NodeInfo = rest.get("/node/info").await?.to_value().await?;
                    node_info.insert(node, (state, Some(res)));
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
            let mut conn = Connection::from(&node);
            match node.open().await {
                Err(e) => {
                    error!("{}", e);
                }
                Ok(_) => {
                    let mut rest = RestClient::from(&mut conn);
                    let node_vms: Vec<Vm> = rest.get("/vm/list").await?.to_value().await?;
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
    pub async fn start_vm(args: StartArgs) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;
        let attach = args.attach;

        // Set node to be queried
        let node: Node;
        if let Some(node_name) = &args.vm_args.node {
            node = config.get_node_by_name(&node_name)?;
        } else {
            node = Node::default();
        }

        if args.vm_args.uuid.is_some() || args.vm_args.id.is_some() || args.vm_args.name.is_some() {
            let mut conn = Connection::from(&node);
            let mut rest = RestClient::from(&mut conn);

            let vm: Vec<Vm> = rest
                .put("/vm/start", Some(args.vm_args.clone()))
                .await?
                .to_value()
                .await?;

            let res = format!("started vm: on node");
            info!("{}", res);
        }
        Ok(())
    }
    pub async fn get_vm_info(args: VmArgs) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;

        // Set node to be queried
        let node: Node;
        if let Some(node_name) = &args.node {
            node = config.get_node_by_name(&node_name)?;
        } else {
            node = Node::default();
        }
        if args.uuid.is_some() || args.id.is_some() || args.name.is_some() {
            match node.open().await {
                Err(e) => {
                    error!("{}", e);
                }
                Ok(mut conn) => {
                    let mut rest = RestClient::from(&mut conn);
                    let vm: Vec<Vm> = rest
                        .post("/vm/info", Some(args.clone()))
                        .await?
                        .to_value()
                        .await?;
                    conn.close();

                    let res = format!("get info for vm: on node:");
                    info!("{}", res);
                }
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_all_vm() -> Result<()> {
        NodeRestClient::get_all_vm().await?;
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
