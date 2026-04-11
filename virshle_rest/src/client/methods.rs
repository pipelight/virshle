use crate::client::Client;

use crate::commons::{
    CreateManyVmArgs, CreateVmArgs, GetManyVmArgs, GetVmArgs, StartManyVmArgs, StartVmArgs,
};

use virshle_core::{
    config::{Config, Node, UserData, VmTemplate},
    hypervisor::{Vm, VmInfo, VmInfoResponse, VmState, VmTable},
    peer::{HostInfo, NodeInfo, Peer},
};

// Connections and Http
use virshle_network::connection::{Connection, ConnectionHandle, ConnectionState};
use virshle_network::http::{Rest, RestClient};

use bon::bon;
use pipelight_exec::Status;
use rand::seq::IndexedRandom;
use std::cmp::Ordering;
use indexmap::IndexMap;
use uuid::Uuid;

// Error handling
use miette::Result;
use tracing::{error, info, trace, warn, debug};
use virshle_error::{LibError, VirshleError, WrapError};

impl Client {
    /// Retrieves working nodes from configuration
    /// and return a rest api convenience helper.
    pub async fn api(&self) -> Result<Methods, VirshleError> {
        // Generate peer list 
        // and open connection to remote peers.
        let mut peers: IndexMap<String, (Peer, RestClient)> = IndexMap::new();

        for (alias, peer) in self.peers.clone() {
            let mut conn: Connection = peer.clone().try_into()?;

            let state = conn.get_state().await?;
            debug!("Peer {:#?} connection state => {:?}", peer.alias, state);

            let mut client: RestClient = conn.into();
            client.base_url("/api/v1");
            client.ping_url("/api/v1/node/ping");

            debug!("Connecting to virshle http API on peer {:#?}", peer.alias);
            let _ = client.open().await.is_ok();
            let _ = client.ping().await.is_ok();
            debug!("Connected to peer {:#?}", peer.alias);
            // Use node only if connection can be established
            //
            // if client.open().await.is_ok() && client.ping().await.is_ok() {
            //     res.insert(node.alias()?, client);
            // }
            peers.insert(alias, (peer, client));
        }

        Ok(Methods {
            peers,
        })
    }
}
pub struct Methods {
    /// List of node aliases and their associated rest client.
    peers: IndexMap<String, (Peer, RestClient)>,
}
impl Methods {
    pub fn node(&mut self) -> Result<NodeMethods<'_>, VirshleError> {
        if let Some((peer, client)) = self.peers.get("Self") {
            Ok(NodeMethods { 
                api: self,
            })
        }
        else {
            let err = LibError::builder()
                .msg("You can't make operations on local node.")
                .help("The local node \"Self\" is set to passive.")
                .build();
            return Err(err.into());
        }

    }
    pub fn peer(&mut self) -> PeerMethods<'_> {
        PeerMethods { api: self }
    }
    pub fn template(&mut self) -> TemplateMethods<'_> {
        TemplateMethods { api: self }
    }
    pub fn vm(&mut self) -> VmMethods<'_> {
        VmMethods { api: self }
    }
}
pub struct NodeMethods<'a> {
    api: &'a mut Methods,
}
pub struct PeerMethods<'a> {
    api: &'a mut Methods,
}
pub struct TemplateMethods<'a> {
    api: &'a mut Methods,
}
pub struct VmMethods<'a> {
    api: &'a mut Methods,
}

#[bon]
impl NodeMethods<'_> {

    #[builder(finish_fn = exec)]
    pub async fn get_info(&mut self) -> Result<(ConnectionState, Option<NodeInfo>), VirshleError> {
        let (ref peer, ref mut rest) = self.api.peers.get_mut("Self").unwrap();
        Self::_get_info(peer, rest).await
    }
    async fn _get_info(
        peer: &Peer,
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
                let message = format!("Node {:#?} is unreachable.", peer.alias);
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
    pub async fn ping(&mut self, alias: Option<String>) -> Result<bool, VirshleError> {
        let mut res: IndexMap<Peer, bool> = IndexMap::new();
        let (ref peer, ref mut rest) = self.api.peers.get_mut("Self").unwrap();
        Self::_ping(peer, rest).await
    }
    async fn _ping(peer: &Peer, rest: &mut RestClient) -> Result<bool, VirshleError> {
        let res = match rest.ping().await {
            Ok(v) => {
                info!("Node {:#?} is pingable.", peer.alias);
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
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn did(
        &mut self,
        alias: Option<String>,
    ) -> Result<IndexMap<Peer, String>, VirshleError> {
        let mut res = IndexMap::new();
        match alias {
            None => {
                for (peer, rest) in self.api.peers.values_mut() {
                    let did = Self::_did(peer, rest).await?;
                    res.insert(peer.clone(), did);
                }
            }
            Some(alias) => {
                if let Some((peer, rest)) = self.api.peers.get_mut(&alias) {
                    let did = Self::_did(peer, rest).await?;
                    res.insert(peer.clone(), did);
                }
            }
        }
        Ok(res)
    }
    async fn _did(
        peer: &Peer,
        rest: &mut RestClient,
    ) -> Result<String, VirshleError> {
        rest.ping().await?;
        let res = rest.get("/node/id").await?.to_value().await;
        let mut did = "".to_owned();
        match res {
            Ok(v) => did = v,
            Err(_) => {
                error!("Operation not supported on peer {:#?}", peer.alias)
            }

        };
        Ok(did)
    }


    #[builder(finish_fn = exec)]
    pub async fn get_info(
        &mut self,
        alias: Option<String>,
    ) -> Result<IndexMap<Peer, (ConnectionState, Option<NodeInfo>)>, VirshleError> {
        let mut res: IndexMap<Peer, (ConnectionState, Option<NodeInfo>)> = IndexMap::new();
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
        peer: &Peer,
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
                let message = format!("Peer {:#?} is unreachable.", peer.alias);
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
    pub async fn ping(
        &mut self,
        alias: Option<String>,
    ) -> Result<IndexMap<Peer, bool>, VirshleError> {
        let mut res: IndexMap<Peer, bool> = IndexMap::new();
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
                info!("Peer {:#?} is pingable.", peer.alias);
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
    pub fn _self(&mut self) -> Result<(&Peer, &mut RestClient), VirshleError> {
        if let Some((ref peer, ref mut rest)) = self.api.peers.get_mut("Self") {
            Ok((peer, rest))
        }
        else {
            let message = format!("Couldn't get the local \"Self\" peer.");
            let help = "";
            let err = LibError::builder().msg(&message).help(help).build();
            Err(err.into())
        }
    }
    pub fn _first(&mut self) -> Result<(&Peer, &mut RestClient), VirshleError> {
        if let Some((_,(ref peer, ref mut rest))) = self.api.peers.first_mut(){
            Ok((peer, rest))
        }
        else {
            let message = format!("Couldn't get a default peer.");
            let help = "Provide a list of peers in your config under [[peer]].";
            let err = LibError::builder().msg(&message).help(help).build();
            Err(err.into())
        }

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
    // If no alias provided:
    // - local node active: return the local node if it is active,
    // - local node passive: returns the first of the config peer list.
    #[builder(
        finish_fn = exec,
        on(String,into),
        on(Option<String>,into)
    )]
    pub fn alias_or_default(
        &mut self,
        alias: Option<String>,
    ) -> Result<(&Peer, &mut RestClient), VirshleError> {
        match alias {
            None => {
                match self.api.peers.get("Self"){
                    Some(_) => self._self(),
                    None => self._first()
                }
            },
            Some(v) => self.alias(&v),
        }
    }
    // Get random non-saturated node.
    pub async fn random(&mut self) -> Result<Peer, VirshleError> {
        let peers: IndexMap<Peer, (ConnectionState, Option<NodeInfo>)> =
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
        let peers: IndexMap<Peer, (ConnectionState, Option<NodeInfo>)> =
            self.api.peer().get_info().exec().await?;

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
        let peers: IndexMap<Peer, (ConnectionState, Option<NodeInfo>)> =
            self.api.peer().get_info().exec().await?;

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
    pub async fn get(
        &mut self,
        alias: Option<String>,
    ) -> Result<IndexMap<Peer, Vec<VmTemplate>>, VirshleError> {
        let mut res: IndexMap<Peer, Vec<VmTemplate>> = IndexMap::new();
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
    ) -> Result<IndexMap<Peer, bool>, VirshleError> {
        let mut res: IndexMap<Peer, bool> = IndexMap::new();
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
        peer: &Peer,
        rest: &mut RestClient,
        args: CreateVmArgs,
    ) -> Result<bool, VirshleError> {
        if let Some(template_name) = args.template_name.clone() {
            info!(
                "[start] reclaiming resources to create a vm from template {:#?} on node {:#?}",
                template_name,
                peer.alias
            );
            let can_create_vm: bool = rest
                .put("/template/reclaim", Some(args))
                .await?
                .to_value()
                .await?;

            info!(
                "[end] reclaiming resources to create a vm from template {:#?} on node {:#?}",
                template_name,
                peer.alias
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

pub struct VmVmmMethods<'a> {
    api: &'a mut Methods,
}
impl VmMethods<'_> {
    pub fn vmm(&mut self) -> VmVmmMethods {
        VmVmmMethods { api: self.api }
    }
}

#[bon]
impl VmVmmMethods<'_> {
    #[builder(finish_fn = exec)]
    pub async fn info(
        &mut self,
        id: Option<u64>,
        uuid: Option<Uuid>,
        name: Option<String>,
        json: Option<bool>,
        alias: Option<String>,
    ) -> Result<VmInfoResponse, VirshleError> {
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias).exec()?;
        rest.open().await?;
        rest.ping().await?;

        rest.base_url("/api/v1/ch");
        let res: VmInfoResponse = rest
            .post("/vm.info", Some(GetVmArgs { id, name, uuid }))
            .await?
            .to_value()
            .await?;

        Ok(res)
    }
}

#[bon]
impl VmMethods<'_> {
    #[builder(finish_fn = exec)]
    pub async fn get_vsock_path(
        &mut self,
        args: GetVmArgs,
        alias: Option<String>,
    ) -> Result<String, VirshleError> {
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias).exec()?;
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

        // info!("[end] fetched info on a vm on node {:#?}", peer.name);
        Ok(path)
    }
}

pub struct VmGetterMethods<'a> {
    api: &'a mut Methods,
}
impl VmMethods<'_> {
    pub fn get(&mut self) -> VmGetterMethods {
        VmGetterMethods { api: self.api }
    }
}
#[bon]
impl VmGetterMethods<'_> {
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn one(
        &mut self,
        id: Option<u64>,
        uuid: Option<Uuid>,
        name: Option<String>,

        alias: Option<String>,
    ) -> Result<VmTable, VirshleError> {
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter
            .alias_or_default()
            .maybe_alias(alias.clone())
            .exec()?;
        let vm: VmTable= Self::_one(
            peer,
            rest,
            Some(GetVmArgs {
                id,
                uuid,
                name,
            }),
        )
        .await?;
        Ok(vm)
    }
    async fn _one(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<GetVmArgs>,
    ) -> Result<VmTable, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vm: VmTable = rest.post("/vm/info", args.clone())
            .await?
            .to_value()
            .await?;
        Ok(vm)
    }
    /// Get a hashmap/dict of all vms per (reachable) node.
    /// - node: the node name set in the virshle config file.
    /// - node: an optional account uuid.
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn many(
        &mut self,
        state: Option<VmState>,
        account: Option<Uuid>,
        /// Specific peer name.
        alias: Option<String>,
    ) -> Result<IndexMap<Peer, Vec<VmTable>>, VirshleError> {
        let mut res: IndexMap<Peer, Vec<VmTable>> = IndexMap::new();
        match alias {
            None => {
                for (_,(peer, rest)) in &mut self.api.peers {
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
    async fn _many(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<GetManyVmArgs>,
    ) -> Result<Vec<VmTable>, VirshleError> {
        let mut vms: Vec<VmTable> = vec![];
        if rest.ping().await.is_ok() {
            let res = rest.post("/vm/info.many", args.clone()).await?.to_value().await;
            match res {
                Ok(v) => {vms = v},
                Err(_) => {}
            };
        }
        Ok(vms)
    }
}
pub struct VmCreateMethods<'a> {
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
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn one(
        &mut self,
        template: Option<String>,
        user_data: Option<UserData>,
        alias: Option<String>,
    ) -> Result<VmTable, VirshleError> {
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter
            .alias_or_default()
            .maybe_alias(alias.clone())
            .exec()?;
        let res: Result<VmTable, VirshleError> = Self::_one(
            peer,
            rest,
            Some(CreateVmArgs {
                user_data,
                template_name: template.clone(),
            }),
        )
        .await;
        match res {
            Ok(vm) => {
                info!("Created vm {:#?} on node {:#?}.", vm.name, alias);
                debug!("{:#?}", vm);
                Ok(vm)
            }
            Err(e) => {
                error!("Couldn't create vm {:#?} on node {:#?}", template, alias);
                Err(e)
            }
        }
    }
    async fn _one(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<CreateVmArgs>,
    ) -> Result<VmTable, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vm: VmTable = rest
            .put("/vm/create", args.clone())
            .await?
            .to_value()
            .await?;
        Ok(vm)
    }
    /// Create multiple virtual machines on a given node.
    ///
    /// # Arguments
    ///
    /// * `args` - a struct containing node name, vm template name,
    ///     and how many vms to create.
    /// * `vm_config_plus`: additional vm configuration to store in db.
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn many(
        &mut self,
        n: Option<u8>,
        template: Option<String>,
        user_data: Option<UserData>,
        alias: Option<String>,
    ) -> Result<Vec<VmTable>, VirshleError> {
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias.clone()).exec()?;
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
        info!("Created {:#?}/{:#?} on node {:#?}", vms.len(), n, alias);
        Ok(vms)
    }
    pub async fn _many(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<CreateManyVmArgs>,
    ) -> Result<Vec<VmTable>, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vms: Vec<Result<VmTable, VirshleError>> = rest
            .put("/vm/create.many", args.clone())
            .await?
            .to_value()
            .await?;
        let res: Vec<VmTable> = vms.into_iter().filter(|e|e.is_ok()).map(|e|e.unwrap()).collect();
        Ok(res)
    }
}

pub struct VmDeleteMethods<'a> {
    api: &'a mut Methods,
}
impl VmMethods<'_> {
    pub fn delete(&mut self) -> VmDeleteMethods {
        VmDeleteMethods { api: self.api }
    }
}
#[bon]
impl VmDeleteMethods<'_> {
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn one(
        &mut self,
        id: Option<u64>,
        uuid: Option<Uuid>,
        name: Option<String>,

        alias: Option<String>,
    ) -> Result<VmTable, VirshleError> {
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias).exec()?;
        let vm: VmTable = Self::_one(
            peer,
            rest,
            Some(GetVmArgs {
                id,
                uuid,
                name,
            }),
        )
        .await?;
        Ok(vm)
    }
    pub async fn _one(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<GetVmArgs>,
    ) -> Result<VmTable, VirshleError> {
        let vm: VmTable = rest
            .put("/vm/delete", args.clone())
            .await?
            .to_value()
            .await?;
        Ok(vm)
    }
    /// Bulk operation
    /// Delete many virtual machine on a node.
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn many(
        &mut self,
        state: Option<VmState>,
        account: Option<Uuid>,
        alias: Option<String>,
    ) -> Result<IndexMap<Peer, IndexMap<Status, Vec<VmTable>>>, VirshleError> {
        let mut res: IndexMap<Peer, IndexMap<Status,Vec<VmTable>>> = IndexMap::new();
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
    ) -> Result<IndexMap<Status, Vec<VmTable>>, VirshleError> {

        let vms: IndexMap<Status,Vec<VmTable>>= rest
            .put("/vm/delete.many", args.clone())
            .await?
            .to_value()
            .await?;
        Ok(vms)
    }
}

pub struct VmStartMethods<'a> {
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
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn one(
        &mut self,
        id: Option<u64>,
        uuid: Option<Uuid>,
        name: Option<String>,
        user_data: Option<UserData>,
        attach: Option<bool>,
        alias: Option<String>,
    ) -> Result<VmTable, VirshleError> {
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias.clone()).exec()?;
        let res: Result<VmTable, VirshleError> = Self::_one(
            peer,
            rest,
            Some(StartVmArgs {
                id,
                uuid,
                name: name.clone(),
                user_data,
                attach,
            }),
        )
        .await;
        match res {
            Ok(vm) => {
                info!("Started vm {:#?} on node {:#?}.", name, alias);
                Ok(vm)
            }
            Err(e) => {
                error!("Couldn't start vm {:#?} on node {:#?}", name, alias);
                Err(e)
            }
        }
    }
    pub async fn _one(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<StartVmArgs>,
    ) -> Result<VmTable, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vm: VmTable = rest
            .put("/vm/start", args.clone())
            .await?
            .to_value()
            .await?;
        Ok(vm)
    }
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn fresh(
        &mut self,
        id: Option<u64>,
        uuid: Option<Uuid>,
        name: Option<String>,
        user_data: Option<UserData>,
        attach: Option<bool>,
        alias: Option<String>,
    ) -> Result<VmTable, VirshleError> {
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias.clone()).exec()?;
        let res: Result<VmTable, VirshleError> = Self::_fresh(
            peer,
            rest,
            Some(StartVmArgs {
                id,
                uuid,
                name: name.clone(),
                user_data,
                attach,
            }),
        )
        .await;
        match res {
            Ok(vm) => {
                info!("Started *fresh* vm {:#?} on node {:#?}.", name, alias);
                Ok(vm)
            }
            Err(e) => {
                error!("Couldn't *fresh* start vm {:#?} on node {:#?}", name, alias);
                Err(e)
            }
        }
    }
    pub async fn _fresh(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<StartVmArgs>,
    ) -> Result<VmTable, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vm: VmTable = rest
            .put("/vm/fresh", args.clone())
            .await?
            .to_value()
            .await?;
        Ok(vm)
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
    ) -> Result<IndexMap<Peer, IndexMap<Status, Vec<VmTable>>>,VirshleError> {
        let mut res: IndexMap<Peer, IndexMap<Status, Vec<VmTable>>>
        = IndexMap::new();
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
    ) -> Result<IndexMap<Status, Vec<VmTable>>,VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vms: IndexMap<Status,Vec<VmTable>> = rest
            .put("/vm/start.many", args.clone())
            .await?
            .to_value()
            .await?;
        Ok(vms)
    }

}
pub struct VmShutdownMethods<'a> {
    api: &'a mut Methods,
}
impl VmMethods<'_> {
    pub fn shutdown(&mut self) -> VmShutdownMethods {
        VmShutdownMethods { api: self.api }
    }
}
#[bon]
impl VmShutdownMethods<'_> {
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn one(
        &mut self,
        id: Option<u64>,
        uuid: Option<Uuid>,
        name: Option<String>,

        alias: Option<String>,
    ) -> Result<VmTable, VirshleError> {
        let mut method = self.api.peer();
        let mut getter = method.get();
        let (peer, rest) = getter.alias_or_default().maybe_alias(alias).exec()?;
        let res: VmTable = Self::_one(
            peer,
            rest,
            Some(GetVmArgs {
                id,
                uuid,
                name,
            }),
        )
        .await?;
        Ok(res)
    }
    async fn _one(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<GetVmArgs>,
    ) -> Result<VmTable, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vm: VmTable = rest
            .put("/vm/shutdown", args.clone())
            .await?
            .to_value()
            .await?;
        Ok(vm)
    }
    /// Bulk operation
    /// Delete many virtual machine on a node.
    #[builder(finish_fn = exec)]
    pub async fn many(
        &mut self,
        state: Option<VmState>,
        account: Option<Uuid>,
        alias: Option<String>,
    ) -> Result<IndexMap<Peer, IndexMap<Status, Vec<VmTable>>>, VirshleError> {
        let mut res: IndexMap<Peer, IndexMap<Status, Vec<VmTable>>> = IndexMap::new();
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
    async fn _many(
        peer: &Peer,
        rest: &mut RestClient,
        args: Option<GetManyVmArgs>,
    ) -> Result<IndexMap<Status, Vec<VmTable>>, VirshleError> {
        rest.open().await?;
        rest.ping().await?;
        let vms: IndexMap<Status, Vec<VmTable>> = rest
            .put("/vm/shutdown.many", args.clone())
            .await?
            .to_value()
            .await?;
        Ok(vms)
    }
}
