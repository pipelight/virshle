use std::os::fd::AsRawFd;

use pipelight_exec::Process;

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError};

/*
* Shorten an interface name to Unix MAX_LENGTH.
*/
pub fn unix_name(name: &str) -> String {
    let res = if name.len() > 15 {
        name[..15].to_owned()
    } else {
        name.to_owned()
    };
    res
}

/*
* Return tap fd.
*/
pub fn get_fd(name: &str) -> Result<i32, VirshleError> {
    let name = unix_name(name);
    let tap_name = tappers::Interface::new(name)?;

    let mut tap = tappers::Tap::new_named(tap_name)?;
    tap.set_nonblocking(true)?;

    let fd = tap.as_raw_fd() as i32;
    let fd_clone = fd.clone();
    Ok(fd.clone())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::network::ovs::{OvsBridge, OvsInterfaceType};

    #[test]
    fn test_unix_name() -> Result<()> {
        let res = unix_name("vm-sasuke_uchiha-main");
        assert_eq!(&res, "vm-sasuke_uchih");
        Ok(())
    }

    #[test]
    fn test_ovs_get_tap_interfaces() -> Result<()> {
        let taps = OvsBridge::get_vm_switch()?.get_ports_by_type(&OvsInterfaceType::Tap);
        println!("{:#?}", taps);
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
        let taps = OvsBridge::get_vm_switch()?.get_ports_by_type(&OvsInterfaceType::Tap);
        let tap = taps.first().unwrap();
        let name = tap.name.clone();
        let res = get_fd(&name)?;
        println!("fd={:#?}", res);
        Ok(())
    }
}
