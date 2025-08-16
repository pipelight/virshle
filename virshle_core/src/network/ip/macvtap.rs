use crate::network::ip::{get_interfaces, get_main_interface};
use crate::network::ovs::OvsBridge;
use crate::network::utils;

use std::fs::File;
use std::os::fd::{AsFd, AsRawFd, RawFd};

// Process
use crate::exec::exec_cmds;

// Error handling
use log::{error, info};
use miette::{IntoDiagnostic, Result};
use pipelight_exec::Process;
use virshle_error::{LibError, VirshleError, WrapError};

// Bug Fix error
// Error booting VM: VmBoot(DeviceManager(CreateVirtioNet(TapError(IoctlError(2147767506, Os { code: 25, kind: Uncategorized, message: "Inappropriate ioctl for device" })))))
pub fn set_permissions(name: &str) -> Result<(), VirshleError> {
    let name = utils::unix_name(name);

    let mut cmds: Vec<String> = vec![];
    let interfaces = get_interfaces()?;
    let interface = interfaces.iter().find(|e| e.name == name);
    if let Some(interface) = interface {
        let path = format!("/dev/tap{}", &interface.index);
        cmds.push(format!("sudo chown $(echo $USER) {path}"));
        cmds.push(format!("sudo chmod 660 {path}"));
    }
    exec_cmds("network", cmds)?;
    Ok(())
}

pub fn get_fd(name: &str) -> Result<RawFd, VirshleError> {
    let name = utils::unix_name(name);
    let interfaces = get_interfaces()?;
    let interface = interfaces.iter().find(|e| e.name == name);
    match interface {
        Some(interface) => {
            let path = format!("/dev/tap{}", &interface.index);
            let file = File::open(path)?;
            let fd = file.as_raw_fd();
            Ok(fd)
        }
        None => {
            let message = "Couldn't open macvtap device.";
            let help = "";
            Err(LibError::builder().msg(message).help(help).build().into())
        }
    }
}

pub fn get_path(name: &str) -> Result<String, VirshleError> {
    let name = utils::unix_name(name);
    let interfaces = get_interfaces()?;
    let interface = interfaces.iter().find(|e| e.name == name);
    match interface {
        Some(interface) => {
            let path = format!("/dev/tap{}", &interface.index);
            Ok(path)
        }
        None => {
            let message = "Couldn't get macvtap path";
            let help = "";
            Err(LibError::builder().msg(message).help(help).build().into())
        }
    }
}

pub fn create(name: &str) -> Result<(), VirshleError> {
    let vm_bridge = OvsBridge::get_vm_switch()?;
    let vm_bridge_name = vm_bridge.name;
    let main_interface = get_main_interface()?;
    let main_interface_name = main_interface.name;

    let name = utils::unix_name(name);
    let mut cmds: Vec<String> = vec![];

    // Create multiqueue tap device
    #[cfg(debug_assertions)]
    cmds.push(format!(
        "sudo ip link \
            add link {vm_bridge_name} \
            name {name} \
            type macvtap"
    ));
    #[cfg(not(debug_assertions))]
    cmds.push(format!(
        "ip link \
            add link {vm_bridge_name} \
            name {name} \
            type macvtap"
    ));
    exec_cmds("network", cmds)?;

    #[cfg(debug_assertions)]
    set_permissions(&name)?;

    Ok(())
}
