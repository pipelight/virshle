use crate::client::Client;

use crate::commons::{
    CreateManyVmArgs, CreateVmArgs, GetManyVmArgs, GetVmArgs, StartManyVmArgs, StartVmArgs,
};
use crate::commons::{
    NodeDefaultMethods, RestDefaultMethods, TemplateDefaultMethods, VmDefaultMethods,
};

use virshle_core::{
    config::{Config, Node, UserData, VmTemplate},
    hypervisor::{Vm, VmInfo, VmState, VmTable},
    node::{HostInfo, NodeInfo, Peer},
};

// Connections and Http
use virshle_network::connection::{Connection, ConnectionHandle, ConnectionState};
use virshle_network::http::{Rest, RestClient};

use bon::bon;
use rand::seq::IndexedRandom;
use std::cmp::Ordering;
use std::collections::HashMap;
use uuid::Uuid;

// Error handling
use miette::Result;
use tracing::{error, info, trace, warn};
use virshle_error::{LibError, VirshleError, WrapError};

impl Client {
    /// Retrieves working nodes from configuration
    /// and return a rest api convenience helper.
    pub async fn api(&mut self) -> Result<Methods, VirshleError> {
        let mut res: HashMap<String, (Peer, RestClient)> = HashMap::new();
        for peer in Config::get()?.peers()? {
            let conn: Connection = peer.clone().try_into().unwrap();
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
            res.insert(peer.alias()?, (peer, client));
        }

        let node: Peer = Config::get()?.node()?.into();
        let conn: Connection = node.clone().try_into().unwrap();
        let mut client: RestClient = conn.into();
        client.base_url("/api/v1");
        client.ping_url("/api/v1/node/ping");
        client.open().await.is_ok();
        client.ping().await.is_ok();

        Ok(Methods {
            peers: res,
            node: (node, client),
        })
    }
}
struct Methods {
    /// List of node aliases and their associated rest client.
    peers: HashMap<String, (Peer, RestClient)>,
    node: (Peer, RestClient),
}
impl Methods {
    pub fn node(&mut self) -> NodeMethods {
        NodeMethods { api: self }
    }
    pub fn peer(&mut self) -> PeerMethods {
        PeerMethods { api: self }
    }
    pub fn template(&mut self) -> TemplateMethods {
        TemplateMethods { api: self }
    }
    pub fn vm(&mut self) -> VmMethods {
        VmMethods { api: self }
    }
}
struct NodeMethods<'a> {
    api: &'a mut Methods,
}
struct PeerMethods<'a> {
    api: &'a mut Methods,
}
struct TemplateMethods<'a> {
    api: &'a mut Methods,
}
struct VmMethods<'a> {
    api: &'a mut Methods,
}

#[bon]
impl NodeMethods<'_> {
    #[builder(finish_fn = exec)]
    async fn get_info(
        &mut self,
    ) -> Result<HashMap<Peer, (ConnectionState, Option<NodeInfo>)>, VirshleError> {
        let (ref peer, ref mut rest) = self.api.node;
        let mut res: HashMap<Peer, (ConnectionState, Option<NodeInfo>)> = HashMap::new();
        let info = Self::_get_info(peer, rest).await?;
        res.insert(peer.clone(), info);
        Ok(res)
    }
    async fn _get_info(
        node: &Peer,
        rest: &mut RestClient,
    ) -> Result<(ConnectionState, Option<NodeInfo>), VirshleError> {
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
                let message = format!("Node {:#?} is unreachable.", node.alias());
                let err: VirshleError = LibError::builder()
                    .msg("Node {:#?} is unreachable.")
                    .help(&help)
                    .build()
                    .into();
                warn!("{:#?}", err);
            }
        };
        Ok((state, info))
    }
    #[builder(finish_fn = exec)]
    async fn ping(&mut self, alias: Option<String>) -> Result<HashMap<Peer, bool>, VirshleError> {
        let mut res: HashMap<Peer, bool> = HashMap::new();
        let (ref peer, ref mut rest) = self.api.node;
        let bool = Self::_ping(peer, rest).await?;
        res.insert(peer.clone(), bool);
        Ok(res)
    }
    async fn _ping(peer: &Peer, rest: &mut RestClient) -> Result<bool, VirshleError> {
        let res = match rest.ping().await {
            Ok(v) => {
                info!("Node {:#?} is pingable.", peer.alias());
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
        Ok(res.is_ok())
    }
}

#[bon]
impl PeerMethods<'_> {
    #[builder(finish_fn = exec)]
    async fn get_info(
        &mut self,
        alias: Option<String>,
    ) -> Result<HashMap<Peer, (ConnectionState, Option<NodeInfo>)>, VirshleError> {
        let mut res: HashMap<Peer, (ConnectionState, Option<NodeInfo>)> = HashMap::new();
        match alias {
            None => {
                for (node, rest) in self.api.peers.values_mut() {
                    let info = Self::_get_info(node, rest).await?;
                    res.insert(node.clone(), info);
                }
            }
            Some(alias) => {
                if let Some((node, rest)) = self.api.peers.get_mut(&alias) {
                    let info = Self::_get_info(node, rest).await?;
                    res.insert(node.clone(), info);
                }
            }
        }
        Ok(res)
    }
    async fn _get_info(
        node: &Peer,
        rest: &mut RestClient,
    ) -> Result<(ConnectionState, Option<NodeInfo>), VirshleError> {
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
                let message = format!("Peer {:#?} is unreachable.", node.alias());
                let err: VirshleError = LibError::builder()
                    .msg("Peer {:#?} is unreachable.")
                    .help(&help)
                    .build()
                    .into();
                warn!("{:#?}", err);
            }
        };
        Ok((state, info))
    }
    #[builder(finish_fn = exec)]
    async fn ping(&mut self, alias: Option<String>) -> Result<HashMap<Peer, bool>, VirshleError> {
        let mut res: HashMap<Peer, bool> = HashMap::new();
        match alias {
            None => {
                for (peer, rest) in self.api.peers.values_mut() {
                    let bool = Self::_ping(peer, rest).await?;
                    res.insert(peer.clone(), bool);
                }
            }
            Some(alias) => {
                if let Some((peer, rest)) = self.api.peers.get_mut(&alias) {
                    let bool = Self::_ping(peer, rest).await?;
                    res.insert(peer.clone(), bool);
                }
            }
        };
        Ok(res)
    }
    async fn _ping(peer: &Peer, rest: &mut RestClient) -> Result<bool, VirshleError> {
        let res = match rest.ping().await {
            Ok(v) => {
                info!("Peer {:#?} is pingable.", peer.alias());
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
        Ok(res.is_ok())
    }
}

struct PeerGetterMethods<'a> {
    api: &'a mut Methods,
}
impl PeerMethods<'_> {
    pub fn get(&mut self) -> PeerGetterMethods {
        PeerGetterMethods { api: self.api }
    }
}
#[bon]
impl PeerGetterMethods<'_> {
    pub fn default(&mut self) -> Result<(&Peer, &mut RestClient), VirshleError> {
        let (ref peer, ref mut rest) = self.api.node;
        Ok((peer, rest))
    }
    pub fn alias(&mut self, alias: &str) -> Result<(&Peer, &mut RestClient), VirshleError> {
        match self.api.peers.get_mut(alias) {
            Some((peer, rest)) => Ok((peer, rest)),
            None => {
                let message = format!("Couldn't get peer {}", alias);
                let help = "You should list available peers.";
                let err = LibError::builder().msg(&message).help(help).build();
                Err(err.into())
            }
        }
    }
    #[builder(finish_fn = exec)]
    pub fn alias_or_default(
        &mut self,
        alias: Option<String>,
    ) -> Result<(&Peer, &mut RestClient), VirshleError> {
        match alias {
            None => self.default(),
            Some(v) => self.alias(&v),
        }
    }
    // Get random non-saturated node.
    pub async fn random(&mut self) -> Result<Peer, VirshleError> {
        let peers: HashMap<Peer, (ConnectionState, Option<NodeInfo>)> =
            self.api.peer().get_info().exec().await?;

        let mut ref_vec: Vec<&Peer> = vec![];
        for (peer, (state, info)) in &peers {
            if let Some(info) = info {
                // Remove saturated nodes
                if info.get_saturation_index().await? < 1.0 {
                    ref_vec.push(peer)
                }
            }
        }
        match ref_vec.choose(&mut rand::rng()) {
            Some(peer) => Ok(peer.to_owned().to_owned()),
            None => Err(LibError::builder()
                .msg("Couldn't get a proper node.")
                .help("Nodes unreachable or saturated!")
                .build()
                .into()),
        }
    }

    /// Get random non-saturated node with weight.
    pub async fn load_balance(&mut self) -> Result<Peer, VirshleError> {
        let peers: HashMap<Peer, (ConnectionState, Option<NodeInfo>)> =
            self.api.node().get_info().exec().await?;

        let mut ref_vec: Vec<&Peer> = vec![];
        for (peer, (state, info)) in &peers {
            if let Some(info) = info {
                // Remove saturated nodes
                if info.get_saturation_index().await? < 1.0 {
                    let weighted_vec: Vec<&Peer>;
                    // Add weight to node
                    if let Some(weight) = peer.weight {
                        weighted_vec = std::iter::repeat_n(peer, weight as usize).collect();
                    } else {
                        weighted_vec = vec![peer];
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

    /// Get random non-saturated node by round-robin.
    pub async fn lowest_saturation_index(&mut self) -> Result<Peer, VirshleError> {
        let peers: HashMap<Peer, (ConnectionState, Option<NodeInfo>)> =
            self.api.node().get_info().exec().await?;

        let mut ref_vec: Vec<(f64, &Peer)> = vec![];
        for (peer, (state, info)) in &peers {
            if let Some(info) = info {
                // Remove saturated nodes
                if info.get_saturation_index().await? < 1.0 {
                    let s_index = info.get_saturation_index().await?;
                    ref_vec.push((s_index, &peer));
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
}

#[bon]
impl TemplateMethods<'_> {
    #[builder(finish_fn = exec)]
    async fn get(
        &mut self,
        alias: Option<String>,
    ) -> Result<HashMap<Peer, Vec<VmTemplate>>, VirshleError> {
        let mut res: HashMap<Peer, Vec<VmTemplate>> = HashMap::new();
        match alias {
            None => {
                for (peer, rest) in self.api.peers.values_mut() {
                    let templates = Self::_get(peer, rest).await?;
                    res.insert(peer.clone(), templates);
                }
            }
            Some(alias) => {
                if let Some((peer, rest)) = self.api.peers.get_mut(&alias) {
                    let templates = Self::_get(peer, rest).await?;
                    res.insert(peer.clone(), templates);
                }
            }
        };
        Ok(res)
    }
    async fn _get(peer: &Peer, rest: &mut RestClient) -> Result<Vec<VmTemplate>, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let templates: Vec<VmTemplate> = rest.get("/template/get.many").await?.to_value().await?;
        Ok(templates)
    }
    #[builder(finish_fn = exec)]
    async fn reclaim(
        &mut self,
        template_name: Option<String>,
        user_data: Option<UserData>,
        alias: Option<String>,
    ) -> Result<HashMap<Peer, bool>, VirshleError> {
        let mut res: HashMap<Peer, bool> = HashMap::new();
        match alias {
            None => {
                for (peer, rest) in self.api.peers.values_mut() {
                    let bool = Self::_reclaim(
                        peer,
                        rest,
                        CreateVmArgs {
                            template_name: template_name.clone(),
                            user_data: user_data.clone(),
                        },
                    )
                    .await?;
                    res.insert(peer.clone(), bool);
                }
            }
            Some(alias) => {
                if let Some((peer, rest)) = self.api.peers.get_mut(&alias) {
                    let bool = Self::_reclaim(
                        peer,
                        rest,
                        CreateVmArgs {
                            template_name,
                            user_data,
                        },
                    )
                    .await?;
                    res.insert(peer.clone(), bool);
                }
            }
        };
        Ok(res)
    }
    async fn _reclaim(
        node: &Peer,
        rest: &mut RestClient,
        args: CreateVmArgs,
    ) -> Result<bool, VirshleError> {
        if let Some(template_name) = args.template_name.clone() {
            info!(
                "[start] reclaiming resources to create a vm from template {:#?} on node {:#?}",
                template_name,
                node.alias()
            );
            let can_create_vm: bool = rest
                .put("/template/reclaim", Some(args))
                .await?
                .to_value()
                .await?;

            info!(
                "[end] reclaiming resources to create a vm from template {:#?} on node {:#?}",
                template_name,
                node.alias()
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

struct VmGetterMethods<'a> {
    api: &'a mut Methods,
}
impl VmMethods<'_> {
    pub fn get(&mut self) -> VmGetterMethods {
        VmGetterMethods { api: self.api }
    }
}
#[bon]
impl VmGetterMethods<'_> {
    /// Get a hashmap/dict of all vms per (reachable) node.
    /// - node: the node name set in the virshle config file.
    /// - node: an optional account uuid.
    #[builder(finish_fn = exec)]
    pub async fn many(
        &mut self,
        state: Option<VmState>,
        account: Option<Uuid>,
        /// Specific peer name.
        alias: Option<String>,
    ) -> Result<HashMap<Peer, Vec<VmTable>>, VirshleError> {
        let mut res: HashMap<Peer, Vec<VmTable>> = HashMap::new();
        match alias {
            None => {
                for (peer, rest) in self.api.peers.values_mut() {
                    let vms = Self::_many(
                        peer,
                        rest,
                        Some(GetManyVmArgs {
                            vm_state: state,
                            account_uuid: account,
                        }),
                    )
                    .await?;
                    res.insert(peer.clone(), vms);
                }
            }
            Some(alias) => {
                let mut method = self.api.peer();
                let mut getter = method.get();
                let (peer, rest) = getter.alias(&alias)?;
                let vms = Self::_many(
                    peer,
                    rest,
                    Some(GetManyVmArgs {
                        vm_state: state,
                        account_uuid: account,
                    }),
                )
                .await?;
                res.insert(peer.clone(), vms);
            }
        }
        Ok(res)
    }
    pub async fn _many(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<GetManyVmArgs>,
    ) -> Result<Vec<VmTable>, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vms: Vec<VmTable> = rest.post("/vm/get", args.clone()).await?.to_value().await?;
        Ok(vms)
    }
}
struct VmCreateMethods<'a> {
    api: &'a mut Methods,
}
impl VmMethods<'_> {
    pub fn create(&mut self) -> VmCreateMethods {
        VmCreateMethods { api: self.api }
    }
}
#[bon]
impl VmCreateMethods<'_> {
    /// Create a virtual machine on a given node.
    ///
    /// # Arguments
    ///
    /// * `args` - a struct containing node name and vm template name.
    /// * `vm_config_plus`: additional vm configuration to store in db.
    ///
    #[builder(finish_fn = exec)]
    pub async fn one(
        &mut self,
        template: Option<String>,
        user_data: Option<UserData>,
        alias: Option<String>,
    ) -> Result<HashMap<Peer, Result<VmTable, VirshleError>>, VirshleError> {
        let mut res: HashMap<Peer, Result<VmTable, VirshleError>> = HashMap::new();
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias).exec()?;
        let vm: Result<VmTable, VirshleError> = Self::_one(
            peer,
            rest,
            Some(CreateVmArgs {
                user_data,
                template_name: template,
            }),
        )
        .await;
        res.insert(peer.clone(), vm);
        Ok(res)
    }
    pub async fn _one(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<CreateVmArgs>,
    ) -> Result<VmTable, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vm: Result<VmTable, VirshleError> = rest
            .post("/vm/create", args.clone())
            .await?
            .to_value()
            .await?;
        vm
    }
    /// Create multiple virtual machines on a given node.
    ///
    /// # Arguments
    ///
    /// * `args` - a struct containing node name, vm template name,
    ///     and how many vms to create.
    /// * `vm_config_plus`: additional vm configuration to store in db.
    ///
    #[builder(finish_fn = exec)]
    pub async fn many(
        &mut self,
        n: Option<u8>,
        template: Option<String>,
        user_data: Option<UserData>,
        alias: Option<String>,
    ) -> Result<HashMap<Peer, Vec<Result<VmTable, VirshleError>>>, VirshleError> {
        let mut res: HashMap<Peer, Vec<Result<VmTable, VirshleError>>> = HashMap::new();
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias).exec()?;
        let vms = Self::_many(
            peer,
            rest,
            Some(CreateManyVmArgs {
                ntimes: n,
                user_data,
                template_name: template,
            }),
        )
        .await?;
        res.insert(peer.clone(), vms);
        Ok(res)
    }
    pub async fn _many(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<CreateManyVmArgs>,
    ) -> Result<Vec<Result<VmTable, VirshleError>>, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vms: Vec<Result<VmTable, VirshleError>> = rest
            .post("/vm/create.many", args.clone())
            .await?
            .to_value()
            .await?;
        Ok(vms)
    }
}

struct VmDeleteMethods<'a> {
    api: &'a mut Methods,
}
impl VmMethods<'_> {
    pub fn delete(&mut self) -> VmDeleteMethods {
        VmDeleteMethods { api: self.api }
    }
}
#[bon]
impl VmDeleteMethods<'_> {
    #[builder(finish_fn = exec)]
    pub async fn one(
        &mut self,
        id: Option<u64>,
        uuid: Option<Uuid>,
        name: Option<String>,

        alias: Option<String>,
    ) -> Result<HashMap<Peer, Result<VmTable, VirshleError>>, VirshleError> {
        let mut res: HashMap<Peer, Result<VmTable, VirshleError>> = HashMap::new();
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias).exec()?;
        let vm: Result<VmTable, VirshleError> = Self::_one(
            peer,
            rest,
            Some(GetVmArgs {
                //
                id,
                uuid,
                name,
            }),
        )
        .await;
        res.insert(peer.clone(), vm);
        Ok(res)
    }
    pub async fn _one(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<GetVmArgs>,
    ) -> Result<VmTable, VirshleError> {
        let vm: Result<VmTable, VirshleError> = rest
            .post("/vm/delete", args.clone())
            .await?
            .to_value()
            .await?;
        vm
    }
    /// Bulk operation
    /// Delete many virtual machine on a node.
    #[builder(finish_fn = exec)]
    pub async fn many(
        &mut self,
        state: Option<VmState>,
        account: Option<Uuid>,
        alias: Option<String>,
    ) -> Result<HashMap<Peer, Vec<Result<VmTable, VirshleError>>>, VirshleError> {
        let mut res: HashMap<Peer, Vec<Result<VmTable, VirshleError>>> = HashMap::new();
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias).exec()?;
        let vms = Self::_many(
            peer,
            rest,
            Some(GetManyVmArgs {
                vm_state: state,
                account_uuid: account,
            }),
        )
        .await?;
        res.insert(peer.clone(), vms);
        Ok(res)
    }
    pub async fn _many(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<GetManyVmArgs>,
    ) -> Result<Vec<Result<VmTable, VirshleError>>, VirshleError> {
        let vms: Vec<Result<VmTable, VirshleError>> = rest
            .post("/vm/delete.many", args.clone())
            .await?
            .to_value()
            .await?;
        Ok(vms)
    }
}

struct VmStartMethods<'a> {
    api: &'a mut Methods,
}
impl VmMethods<'_> {
    pub fn start(&mut self) -> VmStartMethods {
        VmStartMethods { api: self.api }
    }
}
#[bon]
impl VmStartMethods<'_> {
    /// Start a virtual machine on a node.
    #[builder(finish_fn = exec)]
    pub async fn one(
        &mut self,
        id: Option<u64>,
        uuid: Option<Uuid>,
        name: Option<String>,
        user_data: Option<UserData>,
        alias: Option<String>,
    ) -> Result<HashMap<Peer, Result<VmTable, VirshleError>>, VirshleError> {
        let mut res: HashMap<Peer, Result<VmTable, VirshleError>> = HashMap::new();
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias).exec()?;
        let vm: Result<VmTable, VirshleError> = Self::_one(
            peer,
            rest,
            Some(StartVmArgs {
                id,
                uuid,
                name,
                user_data,
            }),
        )
        .await;
        res.insert(peer.clone(), vm);
        Ok(res)
    }
    pub async fn _one(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<StartVmArgs>,
    ) -> Result<VmTable, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vm: Result<VmTable, VirshleError> = rest
            .post("/vm/start", args.clone())
            .await?
            .to_value()
            .await?;
        vm
    }
    /// Bulk operation
    /// Start many virtual machine on a node.
    #[builder(finish_fn = exec)]
    pub async fn many(
        &mut self,
        state: Option<VmState>,
        account: Option<Uuid>,
        user_data: Option<UserData>,
        alias: Option<String>,
    ) -> Result<HashMap<Peer, Vec<Result<VmTable, VirshleError>>>, VirshleError> {
        let mut res: HashMap<Peer, Vec<Result<VmTable, VirshleError>>> = HashMap::new();
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias).exec()?;
        let vms = Self::_many(
            peer,
            rest,
            Some(StartManyVmArgs {
                vm_state: state,
                account_uuid: account,
                user_data,
            }),
        )
        .await?;
        res.insert(peer.clone(), vms);
        // log_response("start", &node.name, &res)?;
        Ok(res)
    }
    pub async fn _many(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<StartManyVmArgs>,
    ) -> Result<Vec<Result<VmTable, VirshleError>>, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vms: Vec<Result<VmTable, VirshleError>> = rest
            .post("/vm/start.many", args.clone())
            .await?
            .to_value()
            .await?;
        Ok(vms)
    }
}
struct VmShutdownMethods<'a> {
    api: &'a mut Methods,
}
impl VmMethods<'_> {
    pub fn shutdown(&mut self) -> VmShutdownMethods {
        VmShutdownMethods { api: self.api }
    }
}
#[bon]
impl VmShutdownMethods<'_> {
    #[builder(finish_fn = exec)]
    pub async fn one(
        &mut self,
        id: Option<u64>,
        uuid: Option<Uuid>,
        name: Option<String>,

        alias: Option<String>,
    ) -> Result<HashMap<Peer, Result<VmTable, VirshleError>>, VirshleError> {
        let mut res: HashMap<Peer, Result<VmTable, VirshleError>> = HashMap::new();
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias).exec()?;
        let vm: Result<VmTable, VirshleError> = Self::_one(
            peer,
            rest,
            Some(GetVmArgs {
                //
                id,
                uuid,
                name,
            }),
        )
        .await;
        res.insert(peer.clone(), vm);
        Ok(res)
    }
    pub async fn _one(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<GetVmArgs>,
    ) -> Result<VmTable, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vm: Result<VmTable, VirshleError> = rest
            .post("/vm/shutdown", args.clone())
            .await?
            .to_value()
            .await?;
        vm
    }
    /// Bulk operation
    /// Delete many virtual machine on a node.
    #[builder(finish_fn = exec)]
    pub async fn many(
        &mut self,
        state: Option<VmState>,
        account: Option<Uuid>,
        alias: Option<String>,
    ) -> Result<HashMap<Peer, Vec<Result<VmTable, VirshleError>>>, VirshleError> {
        let mut res: HashMap<Peer, Vec<Result<VmTable, VirshleError>>> = HashMap::new();
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias).exec()?;
        let vms = Self::_many(
            peer,
            rest,
            Some(GetManyVmArgs {
                vm_state: state,
                account_uuid: account,
            }),
        )
        .await?;
        res.insert(peer.clone(), vms);
        Ok(res)
    }
    pub async fn _many(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<GetManyVmArgs>,
    ) -> Result<Vec<Result<VmTable, VirshleError>>, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vms: Vec<Result<VmTable, VirshleError>> = rest
            .post("/vm/shutdown.many", args.clone())
            .await?
            .to_value()
            .await?;
        Ok(vms)
    }
}
