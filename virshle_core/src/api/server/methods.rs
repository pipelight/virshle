use super::Server;

impl Server {
    /*
     * Return the virshle daemon default socket path.
     */
    pub fn get_socket() -> Result<String, VirshleError> {
        let path = format!("{MANAGED_DIR}/virshle.sock");
        Ok(path)
    }
    pub fn get_host() -> Result<(), VirshleError> {
        Ok(())
    }
    async fn get_all_vm() -> Result<String, VirshleError> {
        let vms = serde_json::to_string(&Vm::get_all().await?)?;
        Ok(vms)
    }
    async fn get_all_template() -> Result<String, VirshleError> {
        let config = VirshleConfig::get()?;
        if let Some(template) = config.template {
            let templates = serde_json::to_string(&template.vm)?;
            Ok(templates)
        } else {
            return Err(LibError::builder()
                .msg("No template on node.")
                .help("")
                .build()
                .into());
        }
    }
    /*
     * Get node info (cpu, ram...)
     */
    async fn get_node_info() -> Result<String, VirshleError> {
        let host = NodeInfo::get().await?;
        let info = serde_json::to_string(&host)?;
        Ok(info)
    }

    async fn create_vm(Query(template_name): Query<String>) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;
        let template = config.get_template(&template_name)?;
        let mut vm = Vm::from(&template);
        vm.create().await?;
        Ok(())
    }

    async fn start_vm(Json(params): Json<VmArgs>) -> Result<Vm, VirshleError> {
        // println!("{:#?}", params);
        if let Some(id) = params.id {
            let mut vm = Vm::get_by_id(&id).await?;
            vm.start().await?;
            Ok(vm)
        } else if let Some(name) = params.name {
            let mut vm = Vm::get_by_name(&name).await?;
            vm.start().await?;
            Ok(vm)
        } else if let Some(uuid) = params.uuid {
            let mut vm = Vm::get_by_uuid(&uuid).await?;
            vm.start().await?;
            Ok(vm)
        } else {
            let message = format!("Couldn't find vm.");
            let help = format!("Are you sure the vm exists on this node?");
            Err(LibError::builder().msg(&message).help(&help).build().into())
        }
    }
    async fn get_vm_info(Json(params): Json<VmArgs>) -> Result<VmInfoResponse, VirshleError> {
        // println!("{:#?}", params);
        if let Some(id) = params.id {
            let mut vm = Vm::get_by_id(&id).await?;
            let info = vm.get_info().await?;
            Ok(info)
        } else if let Some(name) = params.name {
            let mut vm = Vm::get_by_name(&name).await?;
            let info = vm.get_info().await?;
            Ok(info)
        } else if let Some(uuid) = params.uuid {
            let mut vm = Vm::get_by_uuid(&uuid).await?;
            let info = vm.get_info().await?;
            Ok(info)
        } else {
            let message = format!("Couldn't find vm.");
            let help = format!("Are you sure the vm exists on this node?");
            Err(LibError::builder().msg(&message).help(&help).build().into())
        }
    }
    async fn stop_vm(Query(params): Query<HashMap<String, String>>) -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;
        if let Some(id) = params.get("id") {
            let vm = Vm::get_by_id(&id.parse()?).await?;
            vm.shutdown().await?;
        } else if let Some(name) = params.get("name") {
            let vm = Vm::get_by_name(&name).await?;
            vm.shutdown().await?;
        } else if let Some(uuid) = params.get("uuid") {
            let vm = Vm::get_by_uuid(&Uuid::parse_str(uuid)?).await?;
            vm.shutdown().await?;
        }
        Ok(())
    }
}
