mod convert;
mod getters;
mod translate;

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
            let message = "Ovs command failed: Couldn't create tap port";
            let help = format!("{}\n{} ", stderr, &res.io.stdin.unwrap());
            return Err(LibError::builder().msg(message).help(&help).build().into());
        }
        Ok(())
    }
    /*
     * Add vm port into ovs config.
     */
    pub fn create_dpdk_port(&self, name: &str, socket_path: &str) -> Result<(), VirshleError> {
        let vm_bridge_name = &self.name;

        #[cfg(debug_assertions)]
        let cmd = format!(
            "sudo ovs-vsctl \
                -- --may-exist add-port {vm_bridge_name} {name} \
                -- set interface {name} type=dpdkvhostuserclient \
                -- set interface {name} options:vhost-server-path={socket_path} options:n_rxq=2 options:mtu=1500"
        );
        #[cfg(not(debug_assertions))]
        let cmd = format!(
            "ovs-vsctl \
                -- --may-exist add-port {vm_bridge_name} {name} \
                -- set interface {name} type=dpdkvhostuserclient \
                -- set interface {name} options:vhost-server-path={socket_path} options:n_rxq=2 options:mtu=1500"
        );
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        if let Some(stderr) = res.io.stderr {
            let message = "Ovs command failed: Couldn't create dpdk port";
            let help = format!("{}\n{} ", stderr, &res.io.stdin.unwrap());
            return Err(LibError::builder().msg(message).help(&help).build().into());
        }
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
                let (vm_name, net_name) = network_fullname.split_once("-").unwrap();
                Ok(vm_name.to_owned())
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

        let cmd = format!(
            "sudo ovs-vsctl \
            -- --if-exists del-port {} {}",
            self.bridge.name, self.name
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
        let bridges = OvsBridge::get_all()?;
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
        let bridges = Self::get_all()?;

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

        #[cfg(debug_assertions)]
        let cmd = format!(
            "sudo ovs-vsctl \
            -- --may-exist add-br {vm_bridge_name} \
            -- set bridge {vm_bridge_name} datapath_type=netdev"
        );
        #[cfg(not(debug_assertions))]
        let cmd = format!(
            "ovs-vsctl \
            -- --may-exist add-br {vm_bridge_name} \
            -- set bridge {vm_bridge_name} datapath_type=netdev"
        );

        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        if let Some(stderr) = res.io.stderr {
            let message = "Ovs command failed, couldn't create a vm switch.";
            let help = format!("{}\n{} ", stderr, &res.io.stdin.unwrap());

            return Err(LibError::builder().msg(message).help(&help).build().into());
        }

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

        #[cfg(debug_assertions)]
        let mut cmd = format!("sudo ovs-vsctl");
        #[cfg(not(debug_assertions))]
        let mut cmd = format!("ovs-vsctl");

        for port in &self.ports {
            // Only if port is managed by virshle
            if let Some(vm_name) = port.get_vm_name().ok() {
                if !vms_name.contains(&vm_name) {
                    if let Some(_type) = &port.interface._type {
                        match _type {
                            OvsInterfaceType::DpdkVhostUserClient => {
                                cmd += &format!(
                                    " -- --if-exists del-port {} {}",
                                    self.name, port.name
                                );
                                let mut proc = Process::new();
                                let res = proc.stdin(&cmd).run()?;
                            }
                            _ => {}
                        };
                    }
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

    // - add patch cable to main switch (1/2)
    // - add vm switch
    // - add patch cable to vm switch (2/2)
    #[cfg(debug_assertions)]
        let cmd = format!(
            "sudo ovs-vsctl \
            -- --may-exist add-port {main_bridge_name} patch_{main_bridge_name}_{vm_bridge_name} \
            -- set interface patch_{main_bridge_name}_{vm_bridge_name} type=patch options:peer=patch_{vm_bridge_name}_{main_bridge_name} \
            -- --may-exist add-br {vm_bridge_name} \
            -- set bridge {vm_bridge_name} datapath_type=netdev \
            -- --may-exist add-port {vm_bridge_name} patch_{vm_bridge_name}_{main_bridge_name} \
            -- set interface patch_{vm_bridge_name}_{main_bridge_name} type=patch options:peer=patch_{main_bridge_name}_{vm_bridge_name}"
        );
    #[cfg(not(debug_assertions))]
        let cmd = format!(
            "ovs-vsctl \
            -- --may-exist add-port {main_bridge_name} patch_{main_bridge_name}_{vm_bridge_name} \
            -- set interface patch_{main_bridge_name}_{vm_bridge_name} type=patch options:peer=patch_{vm_bridge_name}_{main_bridge_name} \
            -- --may-exist add-br {vm_bridge_name} \
            -- set bridge {vm_bridge_name} datapath_type=netdev \
            -- --may-exist add-port {vm_bridge_name} patch_{vm_bridge_name}_{main_bridge_name} \
            -- set interface patch_{vm_bridge_name}_{main_bridge_name} type=patch options:peer=patch_{main_bridge_name}_{vm_bridge_name}"
        );

    let mut proc = Process::new();
    let res = proc.stdin(&cmd).run()?;

    if let Some(stderr) = res.io.stderr {
        let message = "Ovs command failed, couldn't add internet connectivity to vm switch.";
        let help = format!("{}\n{} ", stderr, &res.io.stdin.unwrap());

        return Err(LibError::builder().msg(message).help(&help).build().into());
    }

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
