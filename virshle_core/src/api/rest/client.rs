use std::collections::HashMap;

// Connections and Http
use crate::connection::{Connection, ConnectionHandle, ConnectionState};
use crate::display::vm::VmTable;
use crate::http_request::{Rest, RestClient};

use crate::cli::{CreateArgs, StartArgs, VmArgs};
use crate::cloud_hypervisor::UserData;
use crate::{Node, NodeInfo, Vm, VmState, VmTemplate};
use std::str::FromStr;

// Error handling
use log::{error, info, trace, warn};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

// Hypervisor
use crate::config::VirshleConfig;

pub mod template {
    use super::*;

    pub async fn get_all() -> Result<HashMap<Node, Vec<VmTemplate>>, VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut templates: HashMap<Node, Vec<VmTemplate>> = HashMap::new();
        for node in nodes {
            let mut conn = Connection::from(&node);
            let mut rest = RestClient::from(&mut conn);
            match rest.open().await {
                Err(e) => {
                    warn!("{}", e);
                }
                Ok(_) => {
                    let node_templates: Vec<VmTemplate> =
                        rest.get("/template/list").await?.to_value().await?;
                    templates.insert(node, node_templates);
                }
            }
        }
        Ok(templates)
    }
}
pub mod node {
    use super::*;

    pub async fn get_info(
    ) -> Result<HashMap<Node, (ConnectionState, Option<NodeInfo>)>, VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut node_info: HashMap<Node, (ConnectionState, Option<NodeInfo>)> = HashMap::new();
        for node in nodes {
            let mut conn = Connection::from(&node);
            let mut rest = RestClient::from(&mut conn);
            match rest.open().await {
                Err(e) => {
                    error!("{}", e);
                    node_info.insert(node, (rest.connection.get_state().await?, None));
                }
                Ok(_) => {
                    let state = rest.connection.get_state().await?;
                    let res: NodeInfo = rest.get("/node/info").await?.to_value().await?;
                    node_info.insert(node, (state, Some(res)));
                }
            }
        }
        Ok(node_info)
    }
}
pub mod vm {
    use super::*;
    /*
     * Get a hashmap/dict of all vms per (reachable) node.
     */
    pub async fn get_all(args: &VmArgs) -> Result<HashMap<Node, Vec<VmTable>>, VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut vms: HashMap<Node, Vec<VmTable>> = HashMap::new();
        for node in nodes {
            let mut conn = Connection::from(&node);
            let mut rest = RestClient::from(&mut conn);
            match rest.open().await {
                Err(e) => {
                    error!("{}", e);
                }
                Ok(_) => {
                    let node_vms: Vec<VmTable> = rest
                        .post("/vm/list", Some(args.clone()))
                        .await?
                        .to_value()
                        .await?;

                    vms.insert(node, node_vms);
                }
            }
        }
        Ok(vms)
    }
    /*
     * Create a virtual machine on a node.
     */
    pub async fn create(args: &CreateArgs) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;
        // Set node to be queried
        let node: Node;
        if let Some(node_name) = &args.node {
            node = config.get_node_by_name(&node_name)?;
        } else {
            node = Node::default();
        }

        // Create a vm from strict definition in file.
        if let Some(file) = &args.file {
            // let mut vm = Vm::from_file(&file)?;
            // vm.create().await?;
        }
        // Create a vm from template.
        if let Some(name) = &args.template {
            let mut conn = Connection::from(&node);
            let mut rest = RestClient::from(&mut conn);

            let vm: Vec<Vm> = rest.put("/vm/create", Some(args)).await?.to_value().await?;
            let vm = vm.first().unwrap();

            let res = format!("Created vm {:#?} on node {:#?}", vm.name, node.name);
            info!("{}", res);
        }

        Ok(())
    }
    pub async fn delete(args: &VmArgs) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;

        // Set node to be queried
        let node: Node;
        if let Some(node_name) = &args.node {
            node = config.get_node_by_name(&node_name)?;
        } else {
            node = Node::default();
        }

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);

        let vm: Vec<Vm> = rest
            .put("/vm/delete", Some(args.clone()))
            .await?
            .to_value()
            .await?;
        let vm = vm.first().unwrap();

        let res = format!("Deleted vm {:#?} on node {:#?}", vm.name, node.name);
        info!("{}", res);

        Ok(())
    }
    /*
     * Start a virtual machine on a node.
     */
    pub async fn start(args: &VmArgs, user_data: Option<UserData>) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;

        // Set node to be queried
        let node: Node;
        if let Some(node_name) = &args.node {
            node = config.get_node_by_name(&node_name)?;
        } else {
            node = Node::default();
        }

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);

        let vm: Vec<Vm> = rest
            .put("/vm/start", Some((args.clone(), user_data)))
            .await?
            .to_value()
            .await?;
        let vm = vm.first().unwrap();

        let res = format!("Started vm {:#?} on node {:#?}", vm.name, node.name);
        info!("{}", res);

        Ok(())
    }
    /*
     * Stop a virtual machine on a node.
     */
    pub async fn shutdown(args: &VmArgs) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;

        // Set node to be queried
        let node: Node;
        if let Some(node_name) = &args.node {
            node = config.get_node_by_name(&node_name)?;
        } else {
            node = Node::default();
        }

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);

        let vm: Vec<Vm> = rest
            .put("/vm/shutdown", Some(args.clone()))
            .await?
            .to_value()
            .await?;
        let vm = vm.first().unwrap();

        let res = format!("Shutdown vm {:#?} on node {:#?}", vm.name, node.name);
        info!("{}", res);

        Ok(())
    }
    pub async fn get_info(args: &VmArgs) -> Result<(), VirshleError> {
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
        let args = VmArgs::default();
        vm::get_all(&args).await?;
        Ok(())
    }
    #[tokio::test]
    async fn test_get_all_vm_w_filter() -> Result<()> {
        let args = VmArgs {
            node: Some("default".to_owned()),
            ..Default::default()
        };
        Ok(())
    }
}
