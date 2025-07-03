use std::collections::HashMap;

// Connections and Http
use crate::connection::{Connection, ConnectionHandle, ConnectionState};
use crate::display::vm::VmTable;
use crate::http_request::{Rest, RestClient};

use crate::cli::{CreateArgs, StartArgs, VmArgs};
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

    pub async fn ping(node_name: Option<String>) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;
        let node = if let Some(node_name) = &node_name {
            config.get_node_by_name(&node_name)?
        } else {
            config.get_node_by_name("default")?
        };

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");

        match rest.open().await {
            Err(e) => {
                error!("{}", e);
            }
            Ok(_) => {
                info!("Node {:#?} is pingable.", &node_name);
            }
        }

        Ok(())
    }

    pub async fn get_info(
    ) -> Result<HashMap<Node, (ConnectionState, Option<NodeInfo>)>, VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut node_info: HashMap<Node, (ConnectionState, Option<NodeInfo>)> = HashMap::new();
        for node in nodes {
            let mut conn = Connection::from(&node);
            let mut rest = RestClient::from(&mut conn);
            rest.base_url("/api/v1");
            rest.ping_url("/api/v1/node/ping");

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
    use crate::cloud_hypervisor::VmConfigPlus;

    use crate::api::method::vm::{GetManyVmArgs, GetVmArgs};

    /// Get a hashmap/dict of all vms per (reachable) node.
    /// - node: the node name set in the virshle config file.
    /// - node: an optional account uuid.
    pub async fn get_all(
        node_name: Option<String>,
        vm_state: Option<String>,
        account_uuid: Option<String>,
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

            match rest.open().await {
                Err(e) => {
                    error!("{}", e);
                }
                Ok(_) => {
                    let node_vms: Vec<Vm> = rest
                        .post("/vm/all", Some((vm_state.clone(), account_uuid.clone())))
                        .await?
                        .to_value()
                        .await?;

                    vms.insert(node, node_vms);
                }
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
        args: &CreateArgs,
        vm_config_plus: Option<VmConfigPlus>,
    ) -> Result<Vm, VirshleError> {
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
            rest.base_url("/api/v1");
            rest.ping_url("/api/v1/node/ping");

            let vm: Vm = rest
                .put("/vm/create", Some((args, vm_config_plus)))
                .await?
                .to_value()
                .await?;

            let res = format!("Created vm {:#?} on node {:#?}", vm.name, node.name);
            info!("{}", res);
            Ok(vm)
        } else {
            Err(LibError::builder()
                .msg("Couldn't create a Vm")
                .help("A template name was not provided.")
                .build()
                .into())
        }
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
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");

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
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");

        let vm: Vec<Vm> = rest
            .put("/vm/start", Some((args, user_data)))
            .await?
            .to_value()
            .await?;
        let vm = vm.first().unwrap();

        let res = format!("Started vm {:#?} on node {:#?}", vm.name, node.name);
        info!("{}", res);

        Ok(())
    }

    /// Bulk operation
    /// Stop many virtual machine on a node.
    pub async fn shutdown_many(
        args: GetManyVmArgs,
        node_name: Option<String>,
    ) -> Result<(), VirshleError> {
        let node = Node::unwrap_or_default(node_name).await?;
        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");

        let vm: Vec<Vm> = rest
            .put("/vm/shutdown", Some(args.clone()))
            .await?
            .to_value()
            .await?;

        Ok(())
    }
    /// Stop a virtual machine on a node.
    pub async fn shutdown(args: GetVmArgs, node_name: Option<String>) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;

        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");

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
    pub async fn get_info(args: &VmArgs) -> Result<VmInfo, VirshleError> {
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
                    Err(e)
                }
                Ok(mut conn) => {
                    let mut rest = RestClient::from(&mut conn);
                    rest.base_url("/api/v1");
                    rest.ping_url("/api/v1/node/ping");

                    let res: VmInfo = rest
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
                    conn.close();
                    Ok(res)
                }
            }
        } else {
            let message = format!("Couldn't retrieve vm infos");
            let help = format!("Are you sure the vm exists on this node?");
            Err(LibError::builder().msg(&message).help(&help).build().into())
        }
    }

    pub async fn get_ch_info(args: Option<GetVmArgs>) -> Result<(), VirshleError> {
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
                    rest.base_url("/api/v1");
                    rest.ping_url("/api/v1/node/ping");

                    let vm: Vec<Vm> = rest
                        .post("/vm/ch/info", Some(args.clone()))
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
