use crate::commons::vm_bulk_results_to_hashmap;
use crate::commons::{
    CreateManyVmArgs, CreateVmArgs, GetManyVmArgs, GetVmArgs, StartManyVmArgs, StartVmArgs,
};
use crate::server::Server;

use http_body_util::BodyExt;
use indexmap::IndexMap;

use pipelight_exec::{Finder, Status};
use uuid::Uuid;

// Hypervisor
use virshle_core::{
    config::{Config, DhcpType, Node, UserData, VmTemplate, VmTemplateTable},
    hypervisor::{
        vm::{Vm, VmTable},
        vmm::types::{VmInfoResponse, VmState},
    },
    network::dhcp::KeaDhcp,
    peer::{HostInfo, NodeInfo, Peer},
};

// Connections and Http
use bon::bon;
use virshle_network::{
    connection::{Connection, TcpConnection},
    http::{Rest, RestClient},
};

// Error handling
use miette::{Diagnostic, Result};
use tokio::task::JoinError;
use tracing::{error, info, warn};
use virshle_error::{LibError, VirshleError};

impl Server {
    pub fn api(&self) -> Result<Methods, VirshleError> {
        // let config = self.config.read().unwrap().clone();
        //
        if !self.config.node.passive {
            return Ok(Methods {
                config: self.config.clone(),
            });
        } else {
            let err = LibError::builder()
                .msg("No node running.")
                .help("Create a node first.")
                .build();
            return Err(err.into());
        }
    }
}
impl Methods {
    pub fn node(&self) -> NodeMethods<'_> {
        NodeMethods { api: self }
    }
    pub fn peer(&self) -> PeerMethods<'_> {
        PeerMethods { api: self }
    }
    pub fn template(&self) -> TemplateMethods<'_> {
        TemplateMethods { api: self }
    }
    pub fn vm(&self) -> VmMethods<'_> {
        VmMethods { api: self }
    }
}
#[derive(Clone)]
pub struct Methods {
    config: Config,
}

#[derive(Clone)]
pub struct NodeMethods<'a> {
    api: &'a Methods,
}
#[derive(Clone)]
pub struct PeerMethods<'a> {
    api: &'a Methods,
}
#[derive(Clone)]
pub struct TemplateMethods<'a> {
    api: &'a Methods,
}
#[derive(Clone)]
pub struct VmMethods<'a> {
    api: &'a Methods,
}

impl NodeMethods<'_> {
    /// Test if locally running node "Self" is responding to external requests
    pub async fn ping(&self) -> Result<(), VirshleError> {
        Ok(())
    }
    /// Get info for locally running node "Self".
    pub async fn info(&self) -> Result<NodeInfo, VirshleError> {
        let res = NodeInfo::get().await?;
        Ok(res)
    }
    /// Get decentralized ID for locally running node "Self".
    pub async fn did(&self) -> Result<String, VirshleError> {
        let did = self.api.config.node.did()?;
        // let mut list = IndexMap::new();
        // list.insert(self.node, v);
        Ok(did)
    }
}
impl PeerMethods<'_> {
    pub async fn ping(&self) -> Result<(), VirshleError> {
        Ok(())
    }
    pub async fn info(&self) -> Result<NodeInfo, VirshleError> {
        let res = NodeInfo::get().await?;
        // let mut list = IndexMap::new();
        // list.insert(self.node, v);
        Ok(res)
    }
}

impl TemplateMethods<'_> {
    pub async fn reclaim(&self, args: CreateVmArgs) -> Result<bool, VirshleError> {
        if let Some(name) = &args.template_name {
            let vm_template = self.api.config.template(name)?;
            // let vm_template = VmTemplate::get_by_name(name)?;
            let node = NodeInfo::get().await?;
            let can = node.can_create_vm(&vm_template).await.is_ok();

            Ok(can)
        } else {
            Ok(false)
        }
    }
    pub async fn get_many(&self) -> Result<IndexMap<String, VmTemplate>, VirshleError> {
        Ok(self.api.config.templates.clone())
    }
    pub async fn get_info_many(&self) -> Result<IndexMap<String, VmTemplateTable>, VirshleError> {
        let vm_templates = self.api.config.templates.clone();
        let mut info: IndexMap<String, VmTemplateTable> = IndexMap::new();
        for (name, e) in vm_templates {
            let table = VmTemplateTable::from(&e)?;
            info.insert(name, table);
        }
        Ok(info)
    }
}

pub struct VmGetterMethods<'a> {
    api: &'a Methods,
}
impl VmMethods<'_> {
    pub fn get(&self) -> VmGetterMethods<'_> {
        VmGetterMethods { api: self.api }
    }
}
#[bon]
impl VmGetterMethods<'_> {
    #[builder(finish_fn = exec)]
    pub async fn one(
        &self,
        id: Option<u64>,
        name: Option<String>,
        uuid: Option<Uuid>,
    ) -> Result<VmTable, VirshleError> {
        let vm = Self::_one(GetVmArgs { id, name, uuid }).await?;
        let res = VmTable::from(&vm).await?;
        Ok(res)
    }
    async fn _one(args: GetVmArgs) -> Result<Vm, VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        Ok(vm)
    }
    #[builder(finish_fn = exec)]
    pub async fn many(
        &self,
        state: Option<VmState>,
        account: Option<Uuid>,
    ) -> Result<Vec<VmTable>, VirshleError> {
        let vms = Self::_many(GetManyVmArgs {
            vm_state: state,
            account_uuid: account,
        })
        .await?;
        let res: Vec<VmTable> = VmTable::from_vec(&vms).await?;
        Ok(res)
    }
    async fn _many(args: GetManyVmArgs) -> Result<Vec<Vm>, VirshleError> {
        let vms = Vm::database()
            .await?
            .many()
            .maybe_account_uuid(args.account_uuid)
            .maybe_vm_state(args.vm_state)
            .get()
            .await?;
        Ok(vms)
    }
}

pub struct VmCreateMethods<'a> {
    api: &'a Methods,
}
impl VmMethods<'_> {
    pub fn create(&self) -> VmCreateMethods<'_> {
        VmCreateMethods { api: self.api }
    }
}
#[bon]
impl VmCreateMethods<'_> {
    #[builder(
        finish_fn = exec,
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn one(
        &self,
        template: Option<String>,
        user_data: Option<UserData>,
    ) -> Result<VmTable, VirshleError> {
        let vm = self
            ._one(CreateVmArgs {
                template_name: template,
                user_data,
            })
            .await?;
        let res = VmTable::from(&vm).await?;
        Ok(res)
    }
    async fn _one(&self, args: CreateVmArgs) -> Result<Vm, VirshleError> {
        match args.template_name {
            Some(name) => {
                let template = self.api.config.template(&name)?;
                let mut vm: Vm = template.try_into()?;

                // Safeguard before creating.
                // Peer::can_create_vm(&template).await?;

                // Warning when no user-data provided.
                match args.user_data {
                    Some(_) => {}
                    None => {
                        let err = LibError::builder()
                            .msg("Couldn't create Vm.")
                            .help("No user_data provided.")
                            .build();
                        warn!("{:#?}", err);
                        // Err(err.into())
                    }
                }

                let vm = vm.create(args.user_data).await?;
                Ok(vm)
            }
            None => Err(LibError::builder()
                .msg("Couldn't create Vm.")
                .help("No valid template provided.")
                .build()
                .into()),
        }
    }
    // TODO: Return Status + Reason
    // Add some reason to the operation state.
    //
    /// Return a VmTable (Vm informations at a specific timestamp).
    /// It is not possible to return a Result through Json, so we return a tuple
    /// of Vm info and the operation State
    #[builder(finish_fn = exec)]
    pub async fn many(
        &self,
        n: Option<u8>,
        template: Option<String>,
        user_data: Option<UserData>,
    ) -> Result<Vec<VmTable>, VirshleError> {
        let vms = self
            ._many(CreateManyVmArgs {
                template_name: template,
                user_data,
                ntimes: n,
            })
            .await?;
        let mut res: Vec<VmTable> = vec![];
        for vm in vms {
            let vm = VmTable::from(&vm).await?;
            res.push(vm);
        }
        Ok(res)
    }
    async fn _many(&self, args: CreateManyVmArgs) -> Result<Vec<Vm>, VirshleError> {
        if args.template_name.is_some() && args.ntimes.is_some() {
            let template = self.api.config.template(&args.template_name.unwrap())?;

            let mut tasks = vec![];
            for i in 0..args.ntimes.unwrap() {
                // Peer::can_create_vm(&template).await?;
                let vm: Vm = template.clone().try_into()?;
                tasks.push(tokio::spawn({
                    let user_data = args.user_data.clone();
                    async move {
                        let mut vm = vm.clone();
                        vm.create(user_data).await
                    }
                }));
            }
            let results: Vec<Result<Result<Vm, VirshleError>, JoinError>> =
                futures::future::join_all(tasks).await;
            let mut res: Vec<Vm> = vec![];
            for result in results {
                match result? {
                    Ok(vm) => res.push(vm),
                    Err(_) => {}
                }
            }
            Ok(res)
        } else {
            Err(LibError::builder()
                .msg("Couldn't create Vm.")
                .help("No valid template provided")
                .build()
                .into())
        }
    }
}

pub struct VmStartMethods<'a> {
    api: &'a Methods,
}
impl VmMethods<'_> {
    pub fn start(&self) -> VmStartMethods<'_> {
        VmStartMethods { api: self.api }
    }
}
#[bon]
impl VmStartMethods<'_> {
    #[builder(
        finish_fn = exec,
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn one(
        &self,
        id: Option<u64>,
        name: Option<String>,
        uuid: Option<Uuid>,
        user_data: Option<UserData>,
        attach: Option<bool>,
        fresh: Option<bool>,
    ) -> Result<VmTable, VirshleError> {
        let vm = Self::_one(StartVmArgs {
            id,
            name,
            uuid,
            user_data,
            attach,
            fresh,
        })
        .await?;
        let res = VmTable::from(&vm).await?;
        Ok(res)
    }
    async fn _one(args: StartVmArgs) -> Result<Vm, VirshleError> {
        let mut vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        vm.start()
            .maybe_user_data(args.user_data.clone())
            .maybe_fresh(args.fresh)
            .exec()
            .await?;
        Ok(vm)
    }
    #[builder(
        finish_fn = exec,
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn provision_ch(
        &self,
        id: Option<u64>,
        name: Option<String>,
        uuid: Option<Uuid>,
    ) -> Result<VmTable, VirshleError> {
        let mut vm = Vm::database()
            .await?
            .one()
            .maybe_id(id)
            .maybe_name(name)
            .maybe_uuid(uuid)
            .get()
            .await?;
        vm.provision_ch_process().await?;
        let res = VmTable::from(&vm).await?;
        Ok(res)
    }
    #[builder(
        finish_fn = exec,
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn create_init_resources(
        &self,
        id: Option<u64>,
        name: Option<String>,
        uuid: Option<Uuid>,
        user_data: Option<UserData>,
    ) -> Result<VmTable, VirshleError> {
        let mut vm = Vm::database()
            .await?
            .one()
            .maybe_id(id)
            .maybe_name(name)
            .maybe_uuid(uuid)
            .get()
            .await?;
        vm.create_init_resources()
            .maybe_user_data(user_data.clone())
            .exec()?;
        let res = VmTable::from(&vm).await?;
        Ok(res)
    }

    #[builder(finish_fn = exec)]
    pub async fn many(
        &self,
        state: Option<VmState>,
        account: Option<Uuid>,
        user_data: Option<UserData>,
    ) -> Result<IndexMap<Status, Vec<VmTable>>, VirshleError> {
        let vms = Vm::database()
            .await?
            .many()
            .maybe_account_uuid(account)
            .maybe_vm_state(state)
            .get()
            .await?;

        let mut tasks = vec![];
        for vm in vms.clone() {
            tasks.push(tokio::spawn({
                let user_data = user_data.clone();
                async move {
                    let mut vm = vm.clone();
                    vm.start().maybe_user_data(user_data).exec().await
                }
            }));
        }
        let results: Vec<Result<Result<Vm, VirshleError>, JoinError>> =
            futures::future::join_all(tasks).await;
        let res: IndexMap<Status, Vec<VmTable>> = vm_bulk_results_to_hashmap(vms, results).await?;
        Ok(res)
    }
}

pub struct VmDeleteMethods<'a> {
    api: &'a Methods,
}
impl VmMethods<'_> {
    pub fn delete(&self) -> VmDeleteMethods<'_> {
        VmDeleteMethods { api: self.api }
    }
}
#[bon]
impl VmDeleteMethods<'_> {
    #[builder(finish_fn = exec)]
    pub async fn one(
        &self,
        id: Option<u64>,
        name: Option<String>,
        uuid: Option<Uuid>,
    ) -> Result<VmTable, VirshleError> {
        let vm = Self::_one(GetVmArgs { id, name, uuid }).await?;
        let res = VmTable::from(&vm).await?;
        Ok(res)
    }
    async fn _one(args: GetVmArgs) -> Result<Vm, VirshleError> {
        let mut vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        vm.delete().await?;
        Ok(vm)
    }

    #[builder(finish_fn = exec)]
    pub async fn many(
        &self,
        state: Option<VmState>,
        account: Option<Uuid>,
    ) -> Result<IndexMap<Status, Vec<VmTable>>, VirshleError> {
        let vms = Vm::database()
            .await?
            .many()
            .maybe_account_uuid(account)
            .maybe_vm_state(state)
            .get()
            .await?;

        let mut tasks = vec![];
        for vm in vms.clone() {
            tasks.push(tokio::spawn({
                async move {
                    let mut vm = vm.clone();
                    vm.delete().await
                }
            }));
        }
        let results: Vec<Result<Result<Vm, VirshleError>, JoinError>> =
            futures::future::join_all(tasks).await;
        let res: IndexMap<Status, Vec<VmTable>> = vm_bulk_results_to_hashmap(vms, results).await?;
        Ok(res)
    }
}

pub struct VmShutdownMethods<'a> {
    api: &'a Methods,
}
impl VmMethods<'_> {
    pub fn shutdown(&self) -> VmShutdownMethods<'_> {
        VmShutdownMethods { api: self.api }
    }
}
#[bon]
impl VmShutdownMethods<'_> {
    #[builder(finish_fn = exec)]
    pub async fn one(
        &self,
        id: Option<u64>,
        name: Option<String>,
        uuid: Option<Uuid>,
    ) -> Result<VmTable, VirshleError> {
        let vm = Self::_one(GetVmArgs { id, name, uuid }).await?;
        let res = VmTable::from(&vm).await?;
        Ok(res)
    }
    async fn _one(args: GetVmArgs) -> Result<Vm, VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        vm.shutdown().await.ok();
        Ok(vm)
    }

    #[builder(finish_fn = exec)]
    pub async fn many(
        &self,
        state: Option<VmState>,
        account: Option<Uuid>,
    ) -> Result<IndexMap<Status, Vec<VmTable>>, VirshleError> {
        let vms = Vm::database()
            .await?
            .many()
            .maybe_account_uuid(account)
            .maybe_vm_state(state)
            .get()
            .await?;

        let mut tasks = vec![];
        for vm in vms.clone() {
            tasks.push(tokio::spawn({
                async move {
                    let vm = vm.clone();
                    vm.shutdown().await
                }
            }));
        }
        let results: Vec<Result<Result<Vm, VirshleError>, JoinError>> =
            futures::future::join_all(tasks).await;
        let res: IndexMap<Status, Vec<VmTable>> = vm_bulk_results_to_hashmap(vms, results).await?;
        Ok(res)
    }
}

impl VmMethods<'_> {
    /*
     * TODO:
     * It should forward vm tty to user tty or ssh session!
     */
    pub async fn _start_attach(
        args: GetVmArgs,
        user_data: Option<UserData>,
    ) -> Result<Vm, VirshleError> {
        let mut vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        Ok(vm)
    }

    /// Attach a virtual machine on a node.
    pub async fn _attach(args: GetVmArgs, node_name: Option<String>) -> Result<(), VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;

        let finder = Finder::new()
            .seed("cloud-hypervisor")
            .seed(&vm.uuid.to_string())
            .search_no_parents()?;

        #[cfg(debug_assertions)]
        if let Some(matches) = finder.matches {
            if let Some(_match) = matches.first() {
                if let Some(pid) = _match.pid {}
            }
        }

        Ok(())
    }
    /// Get detailed information about a VM,
    /// from the underlying cloud-hypervisor process.
    pub async fn get_ch_info(&self, args: GetVmArgs) -> Result<VmInfoResponse, VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        let info = vm.vmm().api()?.info().await?;
        Ok(info.into())
    }
    pub async fn get_raw_ch_info(&self, args: GetVmArgs) -> Result<String, VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        let info = vm.vmm().api()?._info().await?;
        Ok(info.into())
    }

    pub async fn ping_ch(&self, args: GetVmArgs) -> Result<(), VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        vm.vmm().api()?.ping().await
    }
    pub async fn get_vsock_path(&self, args: GetVmArgs) -> Result<String, VirshleError> {
        let vm = Vm::database()
            .await?
            .one()
            .maybe_id(args.id)
            .maybe_name(args.name)
            .maybe_uuid(args.uuid)
            .get()
            .await?;
        let path = vm.get_vsocket()?;
        Ok(path)
    }
}
pub struct VmNetworkMethods<'a> {
    api: &'a Methods,
}
impl VmMethods<'_> {
    pub fn network(&self) -> VmNetworkMethods<'_> {
        VmNetworkMethods { api: self.api }
    }
}
#[bon]
impl VmNetworkMethods<'_> {
    #[builder(finish_fn = exec)]
    pub async fn leases(
        &self,
        id: Option<u64>,
        name: Option<String>,
        uuid: Option<Uuid>,
    ) -> Result<(), VirshleError> {
        match self.api.config.dhcp.clone() {
            Some(DhcpType::Kea(kea_config)) => {
                let mut cli = KeaDhcp::builder().config(kea_config).build().await?;
                let vm = Vm::database()
                    .await?
                    .one()
                    .maybe_id(id)
                    .maybe_name(name)
                    .maybe_uuid(uuid)
                    .get()
                    .await?;
                let ips = cli
                    .ip()
                    .get()
                    .many()
                    .inet4(true)
                    .inet6(true)
                    .vm(vm)
                    .exec()
                    .await?;
            }
            _ => {}
        };
        Ok(())
    }
}

// impl IntoResponse for VmInfoResponse {
//     fn into_response(self) -> axum::response::Response {
//         let json = serde_json::to_string(&self).unwrap();
//         json.into_response()
//     }
// }
// impl IntoResponse for Vm {
//     fn into_response(self) -> axum::response::Response {
//         let json = serde_json::to_string(&self).unwrap();
//         json.into_response()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    fn test_bulk_result_to_response() -> Result<()> {
        Ok(())
    }
}
