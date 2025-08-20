use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, Value::Array};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

// Network primitives
use macaddr::MacAddr6;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Error handling
use miette::{IntoDiagnostic, Result};
use tracing::{error, info, trace, warn};
use virshle_error::{LibError, VirshleError, WrapError};

// Process
use crate::exec::exec_cmds;

use crate::ip::{get_interfaces, get_main_interface};
use crate::network::ovs::OvsBridge;
use crate::network::{interface, utils};

pub fn create(name: &str) -> Result<(), VirshleError> {
    let vm_bridge = OvsBridge::get_vm_switch()?;

    let name = utils::unix_name(name);
    let mut cmds: Vec<String> = vec![];

    // Create multiqueue tap device
    #[cfg(debug_assertions)]
    cmds.push(format!(
        "sudo ip tap \
            add name {name} \
            mode tap"
    ));
    #[cfg(not(debug_assertions))]
    cmds.push(format!(
        "ip tap \
            add name {name} \
            mode tap"
    ));

    // Ensure no ipv6 is configured for tap
    #[cfg(debug_assertions)]
    cmds.push(format!(
        "sudo sysctl \
            net.ipv6.conf.{name}.accept_ra=0"
    ));
    #[cfg(not(debug_assertions))]
    cmds.push(format!(
        "sysctl \
            net.ipv6.conf.{name}.accept_ra=0"
    ));

    exec_cmds("network", cmds)?;
    Ok(())
}

pub fn delete(name: &str) -> Result<(), VirshleError> {
    let vm_bridge_name = "br0";
    let name = utils::unix_name(name);

    let mut cmds: Vec<String> = vec![];
    #[cfg(debug_assertions)]
    cmds.push(format!("sudo ip link del dev {name}"));
    #[cfg(not(debug_assertions))]
    cmds.push(format!("ip link del dev {name}"));

    exec_cmds("network", cmds)?;
    Ok(())
}

pub fn set_mac(name: &str, mac: &MacAddr6) -> Result<(), VirshleError> {
    let mut cmds: Vec<String> = vec![];
    #[cfg(debug_assertions)]
    cmds.push(format!(
        "sudo ip link set dev {} address {}",
        name,
        mac.to_string()
    ));
    #[cfg(not(debug_assertions))]
    cmds.push(format!(
        "ip link set dev {} address {}",
        name,
        mac.to_string()
    ));

    exec_cmds("network", cmds)?;
    Ok(())
}
