use std::collections::HashMap;

// Connections and Http
use crate::connection::{Connection, ConnectionHandle, ConnectionState};
use crate::display::vm::VmTable;
use crate::http_request::{Rest, RestClient};

use crate::cloud_hypervisor::UserData;
use crate::{Node, NodeInfo, Vm, VmInfo, VmState, VmTemplate};
use std::str::FromStr;
use uuid::Uuid;

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
            rest.base_url("/api/v1");
            rest.ping_url("/api/v1/node/ping");

            if rest.open().await.is_ok() && rest.ping().await.is_ok() {
                let node_templates: Vec<VmTemplate> =
                    rest.get("/template/all").await?.to_value().await?;
                templates.insert(node, node_templates);
            }
        }
        Ok(templates)
    }
}
pub mod node {
    use super::*;

    pub async fn ping(node_name: Option<String>) -> Result<(), VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name.clone()).await?;

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;

        match rest.ping().await {
            Err(e) => {
                error!("{}", e);
            }
            Ok(_) => {
                info!("Node {:#?} is pingable.", &node.name);
            }
        }
        Ok(())
    }

    pub async fn get_info(node_name: Option<String>) -> Result<NodeInfo, VirshleError> {
        let node = Node::unwrap_or_default(node_name).await?;

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let res: NodeInfo = rest.get("/node/info").await?.to_value().await?;
        Ok(res)
    }

    pub async fn get_info_all(
    ) -> Result<HashMap<Node, (ConnectionState, Option<NodeInfo>)>, VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut node_info: HashMap<Node, (ConnectionState, Option<NodeInfo>)> = HashMap::new();
        for node in nodes {
            let mut conn = Connection::from(&node);
            let mut rest = RestClient::from(&mut conn);
            rest.base_url("/api/v1");
            rest.ping_url("/api/v1/node/ping");

            if rest.open().await.is_ok() && rest.ping().await.is_ok() {
                let state = rest.connection.get_state().await?;
                let res: NodeInfo = rest.get("/node/info").await?.to_value().await?;
                node_info.insert(node, (state, Some(res)));
            } else {
                node_info.insert(node, (rest.connection.get_state().await?, None));
            }
        }
        Ok(node_info)
    }
}
pub mod vm {
    use super::*;
    use crate::api::{CreateVmArgs, GetManyVmArgs, GetVmArgs};
    use crate::cloud_hypervisor::VmConfigPlus;

    /// Get a hashmap/dict of all vms per (reachable) node.
    /// - node: the node name set in the virshle config file.
    /// - node: an optional account uuid.
    pub async fn get_all(
        args: Option<GetManyVmArgs>,
        node_name: Option<String>,
    ) -> Result<HashMap<Node, Vec<Vm>>, VirshleError> {
        // Single or Multiple node search.
        let config = VirshleConfig::get()?;
        let nodes = if let Some(name) = node_name {
            vec![config.get_node_by_name(&name)?]
        } else {
            config.get_nodes()?
        };

        let mut vms: HashMap<Node, Vec<Vm>> = HashMap::new();

        for node in nodes {
            let mut conn = Connection::from(&node);
            let mut rest = RestClient::from(&mut conn);
            rest.base_url("/api/v1");
            rest.ping_url("/api/v1/node/ping");

            if rest.open().await.is_ok() && rest.ping().await.is_ok() {
                let node_vms: Vec<Vm> =
                    rest.post("/vm/all", args.clone()).await?.to_value().await?;
                vms.insert(node, node_vms);
            }
        }
        Ok(vms)
    }

    /// Bulk
    /// Get a hashmap/dict of all vms per (reachable) node.
    /// - node: the node name set in the virshle config file.
    /// - node: an optional account uuid.
    pub async fn get_info_many(
        args: Option<GetManyVmArgs>,
        node_name: Option<String>,
    ) -> Result<HashMap<Node, Vec<VmTable>>, VirshleError> {
        // Single or Multiple node search.
        let config = VirshleConfig::get()?;
        let nodes = if let Some(name) = node_name {
            vec![config.get_node_by_name(&name)?]
        } else {
            config.get_nodes()?
        };

        let mut vms: HashMap<Node, Vec<VmTable>> = HashMap::new();

        for node in nodes {
            let mut conn = Connection::from(&node);
            let mut rest = RestClient::from(&mut conn);
            rest.base_url("/api/v1");
            rest.ping_url("/api/v1/node/ping");

            if rest.open().await.is_ok() && rest.ping().await.is_ok() {
                let node_vms: Vec<VmTable> = rest
                    .post("/vm/info.many", args.clone())
                    .await?
                    .to_value()
                    .await?;
                vms.insert(node, node_vms);
            }
        }
        Ok(vms)
    }
    /// Create a virtual machine on a given node.
    ///
    /// # Arguments
    ///
    /// * `args` - a struct containing node name and vm template name.
    /// * `vm_config_plus`: additional vm configuration to store in db.
    ///
    pub async fn create(
        args: CreateVmArgs,
        node_name: Option<String>,
        user_data: Option<UserData>,
    ) -> Result<Vm, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;

        // Create a vm from template.
        if let Some(name) = &args.template_name {
            let mut conn = Connection::from(&node);
            let mut rest = RestClient::from(&mut conn);
            rest.base_url("/api/v1");
            rest.ping_url("/api/v1/node/ping");
            rest.open().await?;
            rest.ping().await?;

            let vm: Vm = rest
                .put("/vm/create", Some((args, user_data)))
                .await?
                .to_value()
                .await?;

            info!("Created vm {:#?} on node {:#?}", vm.name, node.name);
            Ok(vm)
        } else {
            Err(LibError::builder()
                .msg("Couldn't create a Vm")
                .help("A template name was not provided.")
                .build()
                .into())
        }
    }

    pub async fn delete(args: GetVmArgs, node_name: Option<String>) -> Result<(), VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let vm: Vm = rest
            .put("/vm/delete", Some(args.clone()))
            .await?
            .to_value()
            .await?;

        info!("Deleted vm {:#?} on node {:#?}", vm.name, node.name);

        Ok(())
    }
    /// Bulk operation
    /// Stop many virtual machine on a node.
    pub async fn delete_many(
        args: GetManyVmArgs,
        node_name: Option<String>,
    ) -> Result<(), VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let vm: Vec<Vm> = rest
            .put("/vm/delete", Some(args.clone()))
            .await?
            .to_value()
            .await?;

        Ok(())
    }
    /// Start a virtual machine on a node.
    pub async fn start(
        args: GetVmArgs,
        node_name: Option<String>,
        user_data: Option<UserData>,
    ) -> Result<Vm, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let vm: Vm = rest
            .put("/vm/start", Some((args, user_data)))
            .await?
            .to_value()
            .await?;

        let res = format!("Started vm {:#?} on node {:#?}", vm.name, node.name);
        info!("{}", res);

        Ok(vm)
    }
    /// Bulk operation
    /// Start many virtual machine on a node.
    pub async fn start_many(
        args: GetManyVmArgs,
        node_name: Option<String>,
        user_data: Option<UserData>,
    ) -> Result<Vec<Vm>, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let vms: Vec<Vm> = rest
            .put("/vm/start.many", Some((args.clone(), user_data.clone())))
            .await?
            .to_value()
            .await?;

        Ok(vms)
    }

    /// Bulk operation
    /// Stop many virtual machine on a node.
    pub async fn shutdown_many(
        args: GetManyVmArgs,
        node_name: Option<String>,
    ) -> Result<Vec<Vm>, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let vms: Vec<Vm> = rest
            .put("/vm/shutdown", Some(args.clone()))
            .await?
            .to_value()
            .await?;

        Ok(vms)
    }
    /// Stop a virtual machine on a node.
    pub async fn shutdown(args: GetVmArgs, node_name: Option<String>) -> Result<Vm, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let vm: Vm = rest
            .put("/vm/shutdown", Some(args.clone()))
            .await?
            .to_value()
            .await?;

        let res = format!("Shutdown vm {:#?} on node {:#?}", vm.name, node.name);
        info!("{}", res);

        Ok(vm)
    }
    pub async fn get_info(
        args: GetVmArgs,
        node_name: Option<String>,
    ) -> Result<VmTable, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let res: VmTable = rest
            .post(
                "/vm/info",
                Some(GetVmArgs {
                    uuid: args.uuid,
                    id: args.id,
                    name: args.name.clone(),
                }),
            )
            .await?
            .to_value()
            .await?;
        Ok(res)
    }

    pub async fn get_ch_info(
        args: GetVmArgs,
        node_name: Option<String>,
    ) -> Result<(), VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let vm: Vm = rest
            .post(
                "/vm/ch/info",
                Some(GetVmArgs {
                    uuid: args.uuid,
                    id: args.id,
                    name: args.name.clone(),
                }),
            )
            .await?
            .to_value()
            .await?;
        conn.close();

        info!("get info for vm: {} on node: {}", vm.name, node.name);

        Ok(())
    }
}
