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

use super::fd;
use crate::network::ovs::OvsBridge;

pub fn delete(name: &str) -> Result<(), VirshleError> {
    let vm_bridge_name = "br0";
    let name = fd::unix_name(name);

    #[cfg(debug_assertions)]
    let cmd = format!("sudo ip link del dev {name}");
    #[cfg(not(debug_assertions))]
    let cmd = format!("ip link del dev {name}");
    let mut proc = Process::new();
    let res = proc.stdin(&cmd).detach().run()?;

    if let Some(stderr) = res.io.stderr {
        let message = format!("ip command failed: {:#?}", cmd);
        let help = format!("{}\n{} ", stderr, &res.io.stdin.unwrap());
        error!("{}", &message);
        error!("{}", &help);
        return Err(LibError::builder().msg(&message).help(&help).build().into());
    }
    Ok(())
}

pub fn create(name: &str) -> Result<(), VirshleError> {
    let vm_bridge = OvsBridge::get_vm_switch()?;

    let name = fd::unix_name(name);
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
        "sudo ip tap \
            add name {name} \
            mode tap"
    ));

    for cmd in cmds {
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        if let Some(stderr) = res.io.stderr {
            let message = format!("ip command failed: {:#?}", cmd);
            let help = format!("{}\n{} ", stderr, &res.io.stdin.unwrap());
            error!("{}", &message);
            error!("{}", &help);
            return Err(LibError::builder().msg(&message).help(&help).build().into());
        }
    }

    Ok(())
}
