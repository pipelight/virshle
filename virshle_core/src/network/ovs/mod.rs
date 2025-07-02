pub mod convert;
mod getters;
mod request;
mod translate;

// traits
use super::interface::Bridge;

// Reexport
pub use translate::{OvsBridge, OvsInterface, OvsInterfaceType, OvsPort};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, Value::Array};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

// Network primitives
use macaddr::MacAddr8;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Error handling
use log::{error, info};
use miette::{IntoDiagnostic, Result};
use pipelight_exec::Process;
use virshle_error::{LibError, VirshleError, WrapError};

// Cloud-hypervisor
use crate::cloud_hypervisor::Vm;
use crate::network::utils;

//Fs
use std::fs;
use std::path::Path;

use crate::network::InterfaceState;

impl OvsBridge {
    /*
     * Add vm port into ovs config.
     */
    pub fn create_tap_port(&self, name: &str) -> Result<(), VirshleError> {
        let vm_bridge_name = &self.name;
        let ifname = utils::unix_name(&name);

        request::OvsRequest::interface(&ifname)
            ._type(request::OvsInterfaceType::System)
            .bridge(vm_bridge_name)
            .create()
            .build()
            .exec()?;

        Ok(())
    }
    /*
     * Add vm port into ovs config.
     */
    pub fn create_dpdk_port(&self, name: &str, socket_path: &str) -> Result<(), VirshleError> {
        let vm_bridge_name = &self.name;

        request::OvsRequest::interface(name)
            ._type(request::OvsInterfaceType::DpdkVhostUserClient)
            .socket_path(socket_path)
            .bridge(vm_bridge_name)
            .create()
            .build()
            .exec()?;

        Ok(())
    }
}

impl OvsPort {
    pub fn is_virshle_port(&self) -> bool {
        self.name.starts_with("vm-")
    }
    pub fn get_vm_name(&self) -> Result<String, VirshleError> {
        match self.is_virshle_port() {
            true => {
                let network_fullname: &str = self.name.strip_prefix("vm-").unwrap();
                if let Some((vm_name, net_name)) = network_fullname.split_once("--") {
                    Ok(vm_name.to_owned())
                } else {
                    let message = "This port is not managed by virshle.";
                    let help = "Port name must be \"vm-<vm-name>--<net-name>\"";
                    Err(LibError::builder().msg(message).help(help).build().into())
                }
            }
            false => {
                let message = "This port is not managed by virshle.";
                let help = "Port name must start by \"vm-\"";
                Err(LibError::builder().msg(message).help(help).build().into())
            }
        }
    }
    /*
     * Remove network port from the vm switch.
     */
    pub fn delete(&self) -> Result<(), VirshleError> {
        let vm_bridge_name = OvsBridge::get_vm_switch()?.name;

        #[cfg(debug_assertions)]
        let mut cmd = format!("sudo ovs-vsctl");
        #[cfg(not(debug_assertions))]
        let mut cmd = format!("ovs-vsctl");

        cmd = format!(
            "{} \
            -- --if-exists del-port {} {}",
            cmd, self.bridge.name, self.name
        );

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        if let Some(stderr) = res.io.stderr {
            let message = "Ovs command failed.";
            let help = format!("{}\n{} ", stderr, &res.io.stdin.unwrap());
            return Err(LibError::builder().msg(message).help(&help).build().into());
        }
        Ok(())
    }
}
impl OvsBridge {
    /*
     * Return the ovs main switch.
     * Usually this switch is the one that provides internet (external)
     * connectivity.
     * It has your main interface (ex: eno1) as port.
     */
    pub fn get_main_switch() -> Result<OvsBridge, VirshleError> {
        let bridges: Vec<OvsBridge> = OvsBridge::get_all()?;
        let bridge = bridges.iter().find(|e| {
            e.ports
                .iter()
                .find(|e| e.interface.name.starts_with("en"))
                .is_some()
        });
        match bridge {
            Some(v) => Ok(v.to_owned()),
            None => {
                let message = "Couldn't identify the main bridge";
                let help = "Did you set up a main virtual switch correctly?";
                return Err(LibError::builder().msg(message).help(help).build().into());
            }
        }
    }
    /*
     * Return the virshle managed switch.
     * This is the switch where the vm are plugged in.
     */
    pub fn get_vm_switch() -> Result<OvsBridge, VirshleError> {
        let vm_bridge_name = "br0";
        let bridges: Vec<OvsBridge> = Self::get_all()?;

        let bridge = bridges.iter().find(|e| {
            e.ports
                .iter()
                .find(|e| e.interface.name == vm_bridge_name)
                .is_some()
        });

        match bridge {
            Some(v) => Ok(v.to_owned()),
            None => {
                let message = "Couldn't identify the vm dedicated bridge";
                let help = "Did you set up a vm virtual switch correctly?";
                return Err(LibError::builder().msg(message).help(help).build().into());
            }
        }
    }
    /*
     * Creates vm dedicated switch to plug vm port in.
     */
    pub fn set_vm_switch() -> Result<(), VirshleError> {
        info!("Create a virtual switch for virtual machines.");
        let vm_bridge_name = "br0";

        request::OvsRequest::bridge(vm_bridge_name)
            ._type(request::OvsBridgeType::System)
            .create()
            .build()
            .exec()?;

        Ok(())
    }

    /*
     * Remove ports from vm dedicated switch,
     * If a port is not related to an existing vm in database.
     */
    pub async fn remove_orphan_ports(&self) -> Result<(), VirshleError> {
        let vms_name: Vec<String> = Vm::get_all()
            .await?
            .iter()
            .map(|e| e.name.to_owned())
            .collect();

        for port in &self.ports {
            // If port is managed by virshle
            if let Some(vm_name) = port.get_vm_name().ok() {
                // If corresponding vm is in database
                if !vms_name.contains(&vm_name) {
                    port.delete()?;
                }
            }
        }
        Ok(())
    }
}
/*
 * Split host main network interface to provide connectivity to vms.
 * see: ./README.md
 */
pub async fn ensure_switches() -> Result<(), VirshleError> {
    // Not fully implemented.
    // Consider there is already a main ovs switch on host
    // and link it to vm switch.

    // Will do the job for now.
    OvsBridge::set_vm_switch()?;
    OvsBridge::get_vm_switch()?.remove_orphan_ports().await?;

    match patch_vm_and_main_switches() {
        Err(e) => {
            error!("{}", e);
        }
        Ok(()) => {}
    }

    info!("Created virshle ovs switches.");
    Ok(())
}

/*
 * Link the vm switch to a main switch (if any) for internet connectivity.
 */
pub fn patch_vm_and_main_switches() -> Result<(), VirshleError> {
    let vm_bridge_name = "br0";
    let main_bridge_name = OvsBridge::get_main_switch()?.name;

    let patch1 = format!("patch_{main_bridge_name}_{vm_bridge_name}");
    let patch2 = format!("patch_{vm_bridge_name}_{main_bridge_name}");

    // - add patch cable to main switch (1/2)
    request::OvsRequest::interface(&patch1)
        ._type(request::OvsInterfaceType::Patch)
        .bridge(&main_bridge_name)
        .peer(&patch2)
        .create()
        .build()
        .exec()?;

    // - add vm switch
    request::OvsRequest::bridge(vm_bridge_name)
        .create()
        .build()
        .exec()?;

    // - add patch cable to vm switch (2/2)
    request::OvsRequest::interface(&patch2)
        .bridge(&vm_bridge_name)
        ._type(request::OvsInterfaceType::Patch)
        .peer(&patch1)
        .create()
        .build()
        .exec()?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    // #[test]
    fn test_ovs_get_main_bridge() -> Result<()> {
        let res = OvsBridge::get_main_switch()?;
        // println!("{:#?}", res);
        Ok(())
    }

    // Create main switch.
    #[tokio::test]
    async fn test_ovs_config_host() -> Result<()> {
        ensure_switches().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_clean_vm_bridge() -> Result<()> {
        OvsBridge::get_vm_switch()?.remove_orphan_ports().await?;
        Ok(())
    }
}
