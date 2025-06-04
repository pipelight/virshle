use super::ip;
use super::ovs;
use super::ovs::{OvsInterface, OvsInterfaceType};
use std::os::fd::AsRawFd;

use pipelight_exec::Process;

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError};

pub fn get_all() -> Result<Vec<OvsInterface>, VirshleError> {
    let interfaces = ovs::interface::get_all()?;
    let taps: Vec<OvsInterface> = interfaces
        .iter()
        .filter(|e| e._type == Some(OvsInterfaceType::Tap))
        .filter(|e| e.name.starts_with("vm-"))
        .map(|e| e.to_owned())
        .collect();
    Ok(taps)
}
pub fn up(name: &str) -> Result<(), VirshleError> {
    let unix_name = name[..15].to_owned();
    ip::device_up(&unix_name)
}
pub fn get_fd(name: &str) -> Result<i32, VirshleError> {
    let unix_name = name[..15].to_owned();
    let tap_name = tappers::Interface::new(unix_name)?;
    let tap = tappers::Tap::new_named(tap_name)?;
    let fd = tap.as_raw_fd() as i32;
    Ok(fd)
}
pub fn create_port(name: &str) -> Result<(), VirshleError> {
    let vm_bridge_name = "br0";
    #[cfg(debug_assertions)]
    let cmd = format!(
        "sudo ip link \
                add link {vm_bridge_name} name {name} \
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ovs_get_interfaces() -> Result<()> {
        let res = get_all()?;
        println!("{:#?}", res);
        Ok(())
    }
    #[test]
    fn test_get_tap_fd() -> Result<()> {
        let res = get_fd("vm-tap1")?;
        println!("fd={:#?}", res);
        Ok(())
    }
    #[test]
    fn test_get_random_tap_fd() -> Result<()> {
        let taps = get_all()?;
        let tap = taps.first().unwrap();
        let name = tap.name.clone();
        let res = get_fd(&name)?;
        println!("fd={:#?}", res);
        Ok(())
    }
}
