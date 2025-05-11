use std::collections::HashMap;

// Http
use crate::cli::LsArgs;
use crate::http_api::Host;
use crate::http_cli::{Connection, HttpRequest, NodeConnection};
use crate::{Node, Vm, VmState, VmTemplate};
use std::str::FromStr;

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
    pub async fn get_all_templates() -> Result<HashMap<Node, Vec<VmTemplate>>, VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut templates: HashMap<Node, Vec<VmTemplate>> = HashMap::new();
        for node in nodes {
            let mut conn = node.open().await?;
            let node_templates: Vec<VmTemplate> =
                conn.get("/template/list").await?.to_value().await?;
            conn.close();
            templates.insert(node, node_templates);
        }
        Ok(templates)
    }
    pub async fn get_node_info() -> Result<HashMap<Node, Host>, VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut node_info: HashMap<Node, Host> = HashMap::new();
        for node in nodes {
            let mut conn = node.open().await?;
            let info: Host = conn.get("/node/info").await?.to_value().await?;
            conn.close();
            node_info.insert(node, info);
        }
        Ok(node_info)
    }
    pub async fn get_all_vm() -> Result<HashMap<Node, Vec<Vm>>, VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut vms: HashMap<Node, Vec<Vm>> = HashMap::new();
        for node in nodes {
            let mut conn = node.open().await?;
            let node_vms: Vec<Vm> = conn.get("/vm/list").await?.to_value().await?;
            conn.close();
            vms.insert(node, node_vms);
        }
        Ok(vms)
    }
    /*
     * Get vms by node.
     */
    pub async fn get_all_vm_w_args(args: LsArgs) -> Result<HashMap<Node, Vec<Vm>>, VirshleError> {
        let config = VirshleConfig::get()?;

        // Parse args
        let nodes: Vec<Node>;
        if let Some(node_name) = &args.node {
            nodes = config
                .get_nodes()?
                .iter()
                .filter(|e| &e.name == node_name)
                .map(|e| e.to_owned())
                .collect();
        } else {
            nodes = config.get_nodes()?;
        }

        let mut vms: HashMap<Node, Vec<Vm>> = HashMap::new();
        for node in nodes {
            let mut conn = node.open().await?;
            let mut node_vms: Vec<Vm> = conn.get("/vm/list").await?.to_value().await?;
            conn.close();

            if let Some(state) = &args.state {
                let state = VmState::from_str(state).unwrap();
                let mut filtered_vms: Vec<Vm> = vec![];
                for vm in node_vms {
                    if vm.get_state().await? == state {
                        filtered_vms.push(vm);
                    }
                }
                node_vms = filtered_vms;
            }
            vms.insert(node, node_vms);
        }

        Ok(vms)
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
}
