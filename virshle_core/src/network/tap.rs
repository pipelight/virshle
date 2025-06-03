use super::ip;
use super::ovs;
use super::ovs::{OvsInterface, OvsInterfaceType};
use std::os::fd::AsRawFd;

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::VirshleError;

pub fn get_all() -> Result<Vec<OvsInterface>, VirshleError> {
    let interfaces = ovs::interface::get_all()?;
    let taps: Vec<OvsInterface> = interfaces
        .iter()
        .filter(|e| e._type == Some(OvsInterfaceType::Tap))
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
