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
    use crate::api::CreateVmArgs;

    pub async fn get_all() -> Result<HashMap<Node, Vec<VmTemplate>>, VirshleError> {
        let nodes = Node::get_all()?;
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
    pub async fn reclaim(
        args: CreateVmArgs,
        node_name: Option<String>,
    ) -> Result<bool, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;

        if let Some(template_name) = args.template_name.clone() {
            info!(
                "[start] reclaiming resources to create a vm from template {:#?} on node {:#?}",
                template_name, node.name
            );
            let mut conn = Connection::from(&node);
            let mut rest = RestClient::from(&mut conn);
            rest.base_url("/api/v1");
            rest.ping_url("/api/v1/node/ping");
            rest.open().await?;
            rest.ping().await?;

            let can_create_vm: bool = rest
                .put("/template/reclaim", Some(args))
                .await?
                .to_value()
                .await?;

            info!(
                "[end] reclaiming resources to create a vm from template {:#?} on node {:#?}",
                template_name, node.name
            );
            Ok(can_create_vm)
        } else {
            Err(LibError::builder()
                .msg("Couldn't reclaim resources for vm creation.")
                .help("A template name was not provided.")
                .build()
                .into())
        }
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

    pub async fn get_info(
        node_name: Option<String>,
    ) -> Result<(Node, (ConnectionState, Option<NodeInfo>)), VirshleError> {
        let node = Node::unwrap_or_default(node_name).await?;

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let state = rest.connection.get_state().await?;
        let info: Option<NodeInfo> = rest.get("/node/info").await?.to_value().await.ok();
        let res = (node, (state, info));

        Ok(res)
    }

    pub async fn get_info_all(
    ) -> Result<HashMap<Node, (ConnectionState, Option<NodeInfo>)>, VirshleError> {
        let nodes = Node::get_all()?;
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
            vec![Node::get_by_name(&name)?]
        } else {
            Node::get_all()?
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
            vec![Node::get_by_name(&name)?]
        } else {
            Node::get_all()?
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
        if let Some(template_name) = args.template_name.clone() {
            info!(
                "[start] creating new vm from template {:#?} on node {:#?}",
                template_name, node.name
            );
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

            info!(
                "[end] created vm {:#?} from template {:#?} on node {:#?}",
                vm.name, template_name, node.name
            );
            Ok(vm)
        } else {
            Err(LibError::builder()
                .msg("Couldn't create a Vm")
                .help("A template name was not provided.")
                .build()
                .into())
        }
    }

    pub async fn delete(args: GetVmArgs, node_name: Option<String>) -> Result<Vm, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;

        info!("[start] deleting a vm on node {:#?}", node.name);

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

        info!("[end] deleted vm {:#?} on node {:#?}", vm.name, node.name);

        Ok(vm)
    }
    /// Bulk operation
    /// Stop many virtual machine on a node.
    pub async fn delete_many(
        args: GetManyVmArgs,
        node_name: Option<String>,
    ) -> Result<Vec<Vm>, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;
        info!("[start] deleting many vms on node {:#?}", node.name);

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let vms: Vec<Vm> = rest
            .put("/vm/delete.many", Some(args.clone()))
            .await?
            .to_value()
            .await?;

        let vms_name = vms
            .iter()
            .map(|e| e.name.to_owned())
            .collect::<Vec<String>>()
            .join(",");
        info!("[end] deleted vms:\n[{vms_name}] on node {:#?}", node.name);

        Ok(vms)
    }
    /// Start a virtual machine on a node.
    pub async fn start(
        args: GetVmArgs,
        node_name: Option<String>,
        user_data: Option<UserData>,
    ) -> Result<Vm, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;
        info!("[start] starting a vm on node {:#?}", node.name);

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

        info!("[end] started vm {:#?} on node {:#?}", vm.name, node.name);

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
        info!("[start] starting many vms on node {:#?}", node.name);

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

        let vms_name = vms
            .iter()
            .map(|e| e.name.to_owned())
            .collect::<Vec<String>>()
            .join(",");
        info!("[end] started vms:\n[{vms_name}] on node {:#?}", node.name);

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
        info!("[start] shutting down many vms on node {:#?}", node.name);

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

        let vms_name = vms
            .iter()
            .map(|e| e.name.to_owned())
            .collect::<Vec<String>>()
            .join(",");
        info!(
            "[end] shutted down vms:\n[{vms_name}] on node {:#?}",
            node.name
        );

        Ok(vms)
    }
    /// Stop a virtual machine on a node.
    pub async fn shutdown(args: GetVmArgs, node_name: Option<String>) -> Result<Vm, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;
        info!("[start] shutting down a vm on node {:#?}", node.name);

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

        info!(
            "[end] shutted down vm {:#?} on node {:#?}",
            vm.name, node.name
        );

        Ok(vm)
    }
    pub async fn get_info(
        args: GetVmArgs,
        node_name: Option<String>,
    ) -> Result<VmTable, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;
        info!("[start] fetching info for on a vm on node {:#?}", node.name);

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let vm: VmTable = rest
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

        info!(
            "[end] fetched info on vm {:#?} on node {:#?}",
            vm.name, node.name
        );

        Ok(vm)
    }

    pub async fn get_ch_info(
        args: GetVmArgs,
        node_name: Option<String>,
    ) -> Result<(), VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;
        info!(
            "[start] fetching CH info for on a vm on node {:#?}",
            node.name
        );

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

        info!(
            "[end] fetched CH info on vm {:#?} on node {:#?}",
            vm.name, node.name
        );

        Ok(())
    }
}
