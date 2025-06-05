pub mod tap {
    use super::*;

    /*
     * Add vm port into ovs config.
     */
    pub fn create_port(name: &str) -> Result<(), VirshleError> {
        let vm_bridge_name = "br0";
        #[cfg(debug_assertions)]
        let cmd = format!(
            "sudo ovs-vsctl \
                -- --may-exist add-port {vm_bridge_name} {name} \
                -- set interface {name} type=tap"
        );
        #[cfg(not(debug_assertions))]
        let cmd = format!(
            "ovs-vsctl \
                -- --may-exist add-port {vm_bridge_name} {name} \
                -- set interface {name} type=tap"
        );
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        if let Some(stderr) = res.io.stderr {
            let message = "Ovs command failed";
            let help = format!("{}\n{} ", stderr, &res.io.stdin.unwrap());
            return Err(LibError::builder().msg(message).help(&help).build().into());
        }
        Ok(())
    }

    /*
     * Return all Tap interface from ovs cli.
     */
    pub fn get_all() -> Result<Vec<OvsInterface>, VirshleError> {
        let interfaces = super::interface::get_all()?;
        let taps: Vec<OvsInterface> = interfaces
            .iter()
            .filter(|e| e._type == Some(OvsInterfaceType::Tap))
            .filter(|e| e.name.starts_with("vm-"))
            .map(|e| e.to_owned())
            .collect();
        Ok(taps)
    }
}
