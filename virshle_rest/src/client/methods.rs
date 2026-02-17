use crate::client::Client;

use crate::commons::{
    NodeDefaultMethods, RestDefaultMethods, TemplateDefaultMethods, VmDefaultMethods,
};

use virshle_core::{
    config::{Config, UserData, VmTemplate},
    hypervisor::{Vm, VmInfo, VmState, VmTable},
    node::{HostInfo, Node, NodeInfo},
};

// Connections and Http
use virshle_network::connection::{Connection, ConnectionHandle, ConnectionState};
use virshle_network::http::{Rest, RestClient};

use bon::bon;
use rand::seq::IndexedRandom;
use std::cmp::Ordering;
use std::collections::HashMap;

// Error handling
use miette::Result;
use tracing::{error, info, trace, warn};
use virshle_error::{LibError, VirshleError, WrapError};

impl Client {
    /// Retrieves working nodes from configuration
    /// and return a rest api convenience helper.
    pub async fn api(&mut self) -> Result<Methods, VirshleError> {
        let nodes = Config::get()?.nodes()?;
        let mut res: HashMap<String, (Node, RestClient)> = HashMap::new();
        for node in nodes {
            let conn: Connection = node.clone().try_into().unwrap();
            let mut client: RestClient = conn.into();
            client.base_url("/api/v1");
            client.ping_url("/api/v1/node/ping");

            client.open().await.is_ok();
            client.ping().await.is_ok();
            // Use node only if connection can be established
            //
            // if client.open().await.is_ok() && client.ping().await.is_ok() {
            //     res.insert(node.alias()?, client);
            // }
            res.insert(node.alias()?, (node, client));
        }
        Ok(Methods { nodes: res })
    }
}
#[derive(Default)]
struct Methods {
    /// List of node aliases and their associated rest client.
    nodes: HashMap<String, (Node, RestClient)>,
}
impl Methods {
    pub fn template(&mut self) -> TemplateMethods {
        TemplateMethods { nodes: &self.nodes }
    }
    pub fn vm(&mut self) -> VmMethods {
        VmMethods { nodes: &self.nodes }
    }
    pub fn node(&mut self) -> NodeMethods {
        NodeMethods { nodes: &self.nodes }
    }
}
#[derive(Clone)]
struct NodeMethods<'a> {
    nodes: &'a HashMap<String, (Node, RestClient)>,
}
#[derive(Clone)]
struct TemplateMethods<'a> {
    nodes: &'a HashMap<String, (Node, RestClient)>,
}
#[derive(Clone)]
struct VmMethods<'a> {
    nodes: &'a HashMap<String, (Node, RestClient)>,
}

#[bon]
// impl NodeDefaultMethods for NodeMethods {
impl NodeMethods {
    #[builder(finish_fn = exec)]
    async fn get_info(
        &mut self,
        alias: Option<String>,
    ) -> Result<HashMap<Node, (ConnectionState, Option<NodeInfo>)>, VirshleError> {
        let mut res = HashMap::new();
        match alias {
            None => {
                for (node, rest) in self.nodes.values_mut() {
                    let (node, info) = Self::_get_info(node, rest).await?;
                    res.insert(node, info);
                }
            }
            Some(alias) => {
                if let Some((node, rest)) = self.nodes.get_mut(&alias) {
                    let (node, info) = Self::_get_info(node, rest).await?;
                    res.insert(node, info);
                }
            }
        }
        Ok(res)
    }
    async fn _get_info(
        node: &Node,
        rest: &mut RestClient,
    ) -> Result<(Node, (ConnectionState, Option<NodeInfo>)), VirshleError> {
        let mut info: Option<NodeInfo> = None;
        let state = rest.connection.get_state().await?;
        match state {
            ConnectionState::DaemonUp => {
                if rest.open().await.is_ok() && rest.ping().await.is_ok() {
                    info = rest.get("/node/info").await?.to_value().await.ok();
                }
            }
            _ => {
                let help = format!("ConnectionState: {}", state.display());
                let err: VirshleError = LibError::builder()
                    .msg("Node {:#?} is unreachable.")
                    .help(&help)
                    .build()
                    .into();
                warn!("{:#?}", err);
            }
        };
        Ok((node.to_owned(), (state, info)))
    }

    #[builder]
    #[builder(finish_fn = exec)]
    async fn ping(&mut self, alias: Option<String>) -> Result<HashMap<Node, bool>, VirshleError> {
        let mut res = HashMap::new();
        match alias {
            None => {
                for (node, rest) in self.nodes.values_mut() {
                    let (node, bool) = Self::_ping(node, rest).await?;
                    res.insert(node, bool);
                }
            }
            Some(alias) => {
                if let Some((node, rest)) = self.nodes.get_mut(&alias) {
                    let (node, bool) = Self::_ping(node, rest).await?;
                    res.insert(node, bool);
                }
            }
        };
        Ok(res)
    }
    async fn _ping(node: &Node, rest: &mut RestClient) -> Result<(Node, bool), VirshleError> {
        let bool = match rest.ping().await {
            Ok(v) => {
                info!("Node {:#?} is pingable.", node.alias());
                Ok(v)
            }
            Err(e) => {
                let state = rest.connection.get_state().await?;
                let help = format!("ConnectionState: {}", state.display());
                let err: VirshleError = WrapError::builder()
                    .origin(e.into())
                    .msg("Node {:#?} did not answer.")
                    .help(&help)
                    .build()
                    .into();
                warn!("{:#?}", err);
                Err(err)
            }
        };
        Ok((node.to_owned(), bool.is_ok()))
    }
}

impl NodeMethods<'_> {
    // Get random non-saturated node.
    pub async fn get_by_random(&mut self) -> Result<Node, VirshleError> {
        let nodes: HashMap<Node, (ConnectionState, Option<NodeInfo>)> =
            self.get_info().exec().await?;

        let mut ref_vec: Vec<&Node> = vec![];
        for (node, (state, info)) in &nodes {
            if let Some(info) = info {
                // Remove saturated nodes
                if info.get_saturation_index().await? < 1.0 {
                    ref_vec.push(&node)
                }
            }
        }
        match ref_vec.choose(&mut rand::rng()) {
            Some(node) => Ok(node.to_owned().to_owned()),
            None => Err(LibError::builder()
                .msg("Couldn't get a proper node.")
                .help("Nodes unreachable or saturated!")
                .build()
                .into()),
        }
    }

    // Get random non-saturated node with weight.
    pub async fn get_by_load_balance(&mut self) -> Result<Node, VirshleError> {
        let nodes: HashMap<Node, (ConnectionState, Option<NodeInfo>)> =
            self.get_info().exec().await?;

        let mut ref_vec: Vec<&Node> = vec![];
        for (node, (state, info)) in &nodes {
            if let Some(info) = info {
                // Remove saturated nodes
                if info.get_saturation_index().await? < 1.0 {
                    let weighted_vec: Vec<&Node>;
                    // Add weight to node
                    if let Some(weight) = node.weight {
                        weighted_vec = std::iter::repeat_n(node, weight as usize).collect();
                    } else {
                        weighted_vec = vec![&node];
                    }
                    ref_vec.extend(weighted_vec);
                }
            }
        }
        match ref_vec.choose(&mut rand::rng()) {
            Some(node_ref) => Ok(node_ref.to_owned().to_owned()),
            None => Err(LibError::builder()
                .msg("Couldn't get a proper node.")
                .help("Nodes unreachable or saturated!")
                .build()
                .into()),
        }
    }

    // Get random non-saturated node by round-robin.
    pub async fn get_by_saturation_index(&mut self) -> Result<Node, VirshleError> {
        let nodes: HashMap<Node, (ConnectionState, Option<NodeInfo>)> =
            self.get_info().exec().await?;

        let mut ref_vec: Vec<(f64, &Node)> = vec![];
        for (node, (state, info)) in &nodes {
            if let Some(info) = info {
                // Remove saturated nodes
                if info.get_saturation_index().await? < 1.0 {
                    let s_index = info.get_saturation_index().await?;
                    ref_vec.push((s_index, &node));
                }
            }
        }
        // Find lowest saturation index.
        ref_vec.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
        ref_vec.first();

        match ref_vec.first() {
            Some((_, node)) => Ok(node.to_owned().to_owned()),
            None => Err(LibError::builder()
                .msg("Couldn't get a proper node.")
                .help("Nodes unreachable or saturated!")
                .build()
                .into()),
        }
    }

    // If self node can create the requested template
    pub async fn can_create_vm(vm_template: &VmTemplate) -> Result<(), VirshleError> {
        let info = HostInfo::get().await?;
        // Check saturation
        if info.disk.is_saturated().await?
            || info.ram.is_saturated().await?
            || info.cpu.is_saturated().await?
        {
            return Err(LibError::builder()
                .msg("Not allowed to create VM: node is saturated.")
                .help("Try deleting unused VMs or change saturation indexes in config.")
                .build()
                .into());
        // Check remaining disk space
        } else if let Some(disks) = &vm_template.disk {
            let disks_total_size: u64 = disks.into_iter().map(|e| e.get_size().unwrap_or(0)).sum();
            if disks_total_size < info.disk.available {
                return Ok(());
            } else {
                let help = format!(
                    "Not enough disk space for new vm from template {:#?}",
                    vm_template.name
                );
                warn!("{}", help);
                return Err(LibError::builder()
                    .msg("Couldn't create Vm")
                    .help(&help)
                    .build()
                    .into());
            }
        } else {
            Ok(())
        }
    }
}

impl TemplateMethods<'_> {
    async fn get_many(&self) -> Result<HashMap<Node, Vec<VmTemplate>>, VirshleError> {
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
    async fn get_info_many(&self) -> Result<HashMap<Node, Vec<VmTemplateTable>>, VirshleError> {
        Ok(templates)
    }
    async fn reclaim(args: CreateVmArgs, node_name: Option<String>) -> Result<bool, VirshleError> {
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

impl VmMethods<'_> {
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
    ) -> Result<VmTable, VirshleError> {
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
            conn.close();

            info!(
                "[end] created vm {:#?} from template {:#?} on node {:#?}",
                vm.name, template_name, node.name
            );
            Ok(VmTable::from(&vm).await?)
        } else {
            Err(LibError::builder()
                .msg("Couldn't create a Vm")
                .help("A template name was not provided.")
                .build()
                .into())
        }
    }
    /// Create a virtual machine on a given node.
    ///
    /// # Arguments
    ///
    /// * `args` - a struct containing node name, vm template name,
    ///     and how many vms to create.
    /// * `vm_config_plus`: additional vm configuration to store in db.
    ///
    pub async fn create_many(
        args: CreateManyVmArgs,
        node_name: Option<String>,
        user_data: Option<UserData>,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError> {
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

            let res: HashMap<Status, Vec<Vm>> = rest
                .put("/vm/create.many", Some((args, user_data)))
                .await?
                .to_value()
                .await?;
            conn.close();

            log_response("create", &node.name, &res)?;
            Ok(res)
        } else {
            Err(LibError::builder()
                .msg("Couldn't create a Vm")
                .help("A template name was not provided.")
                .build()
                .into())
        }
    }

    pub async fn delete(
        args: GetVmArgs,
        node_name: Option<String>,
    ) -> Result<VmTable, VirshleError> {
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
        conn.close();

        info!("[end] deleted vm {:#?} on node {:#?}", vm.name, node.name);

        Ok(VmTable::from(&vm).await?)
    }
    /// Bulk operation
    /// Stop many virtual machine on a node.
    pub async fn delete_many(
        args: GetManyVmArgs,
        node_name: Option<String>,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;
        info!("[start] deleting many vms on node {:#?}", node.name);

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let res: HashMap<Status, Vec<Vm>> = rest
            .put("/vm/delete.many", Some(args.clone()))
            .await?
            .to_value()
            .await?;
        conn.close();

        log_response("delete", &node.name, &res)?;
        Ok(res)
    }
    /// Start a virtual machine on a node.
    pub async fn start(
        args: GetVmArgs,
        node_name: Option<String>,
        user_data: Option<UserData>,
    ) -> Result<VmTable, VirshleError> {
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
        conn.close();

        info!("[end] started vm {:#?} on node {:#?}", vm.name, node.name);

        Ok(VmTable::from(&vm).await?)
    }
    /// Bulk operation
    /// Start many virtual machine on a node.
    pub async fn start_many(
        args: GetManyVmArgs,
        node_name: Option<String>,
        user_data: Option<UserData>,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;
        info!("[start] starting many vms on node {:#?}", node.name);

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let res: HashMap<Status, Vec<Vm>> = rest
            .put("/vm/start.many", Some((args.clone(), user_data.clone())))
            .await?
            .to_value()
            .await?;
        conn.close();

        log_response("start", &node.name, &res)?;
        Ok(res)
    }

    /// Bulk operation
    /// Stop many virtual machine on a node.
    pub async fn shutdown_many(
        args: GetManyVmArgs,
        node_name: Option<String>,
    ) -> Result<HashMap<Status, Vec<Vm>>, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;
        info!("[start] shutting down many vms on node {:#?}", node.name);

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let res: HashMap<Status, Vec<Vm>> = rest
            .put("/vm/shutdown.many", Some(args.clone()))
            .await?
            .to_value()
            .await?;
        conn.close();

        log_response("shutdown", &node.name, &res)?;
        Ok(res)
    }
    /// Stop a virtual machine on a node.
    pub async fn shutdown(
        args: GetVmArgs,
        node_name: Option<String>,
    ) -> Result<VmTable, VirshleError> {
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
        conn.close();

        info!(
            "[end] shutted down vm {:#?} on node {:#?}",
            vm.name, node.name
        );

        Ok(VmTable::from(&vm).await?)
    }
    pub async fn get_definition(
        args: GetVmArgs,
        node_name: Option<String>,
    ) -> Result<Vm, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;
        info!("[start] fetching info for on a vm on node {:#?}", node.name);

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let vm: Vm = rest
            .post(
                "/vm/definition",
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
            "[end] fetched info on vm {:#?} on node {:#?}",
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
        conn.close();

        info!(
            "[end] fetched info on vm {:#?} on node {:#?}",
            vm.name, node.name
        );

        Ok(vm)
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
            } else {
                // Logging
                let state = rest.connection.get_state().await?;
                let message = format!("node {:#?} unreachable", node.name);
                match state {
                    ConnectionState::SshAuthError => {
                        let message = format!("node {:#?} ssh authenticaton rejected", node.name);
                        warn!("{}", &message)
                    }
                    ConnectionState::Unreachable => {
                        let message = format!("node {:#?} is unreachable", node.name);
                        warn!("{}", &message)
                    }
                    ConnectionState::Down => {
                        let message = format!("node {:#?} host is down", node.name);
                        warn!("{}", &message)
                    }
                    ConnectionState::DaemonDown => {
                        let message = format!("node {:#?} daemon is down", node.name);
                        warn!("{}", &message)
                    }
                    ConnectionState::SocketNotFound => {
                        let message = format!("node {:#?} no socket found", node.name);
                        warn!("{}", &message)
                    }
                    _ => {}
                };
            }
        }
        Ok(vms)
    }

    pub async fn get_raw_ch_info(
        args: GetVmArgs,
        node_name: Option<String>,
    ) -> Result<String, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;
        info!("[start] fetching CH info for a vm on node {:#?}", node.name);

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        rest.base_url("/api/v1/ch");
        let res: String = rest
            .post(
                "/vm.info.raw",
                Some(GetVmArgs {
                    uuid: args.uuid,
                    id: args.id,
                    name: args.name.clone(),
                }),
            )
            .await?
            .to_string()
            .await?;

        conn.close();
        info!("[end] fetched CH info for a vm on node {:#?}", node.name);

        Ok(res)
    }

    pub async fn get_ch_info(
        args: GetVmArgs,
        node_name: Option<String>,
    ) -> Result<VmInfoResponse, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;
        info!("[start] fetching CH info for a vm on node {:#?}", node.name);

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        rest.base_url("/api/v1/ch");
        let res: VmInfoResponse = rest
            .post(
                "/vm.info",
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

        info!("[end] fetched CH info for a vm on node {:#?}", node.name);
        Ok(res)
    }
    pub async fn get_vsock_path(
        args: GetVmArgs,
        node_name: Option<String>,
    ) -> Result<String, VirshleError> {
        // Set node to be queried
        let node = Node::unwrap_or_default(node_name).await?;
        info!("[start] fetching info for on a vm on node {:#?}", node.name);

        let mut conn = Connection::from(&node);
        let mut rest = RestClient::from(&mut conn);
        rest.base_url("/api/v1");
        rest.ping_url("/api/v1/node/ping");
        rest.open().await?;
        rest.ping().await?;

        let path: String = rest
            .post(
                "/vm/get_vsock_path",
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

        info!("[end] fetched info on a vm on node {:#?}", node.name);
        Ok(path)
    }

    /// Log response
    pub fn log_response(
        tag: &str,
        node: &str,
        response: &HashMap<Status, Vec<Vm>>,
    ) -> Result<(), VirshleError> {
        let tag = format!("[{tag}]");
        for (k, v) in response.iter() {
            match k {
                Status::Failed => {
                    let tag = tag.red();
                    let vms_name: Vec<String> = v.iter().map(|e| e.name.to_owned()).collect();
                    let vms_name = vms_name.join(" ");
                    info!("{tag} failed for vms [{}] on node {node}", vms_name);
                }
                Status::Succeeded => {
                    let tag = tag.green();
                    let vms_name: Vec<String> = v.iter().map(|e| e.name.to_owned()).collect();
                    let vms_name = vms_name.join(" ");
                    info!("{tag} succedded for vms [{}] on node {node}", vms_name);
                }
                _ => {}
            }
        }
        Ok(())
    }
}
