use super::ip::IpInterface;
use super::ovs;
use super::ovs::{OvsBridge, OvsInterface};

use bon::{bon, Builder};

use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, Value::Array};

// Error handling
use log::{error, info};
use miette::{IntoDiagnostic, Result};
use pipelight_exec::Process;
use virshle_error::{LibError, VirshleError, WrapError};

// pub trait Interface {
//     fn create() -> Result<Self, VirshleError>;
//     fn delete() -> Result<Self, VirshleError>;
// }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InterfaceState {
    Up,
    Down,
}
impl InterfaceState {
    fn from_str(s: &str) -> Result<Option<InterfaceState>> {
        let cased = s.to_case(Case::Title);
        let res = match cased.as_str() {
            "Up" => Some(InterfaceState::Up),
            "Down" => Some(InterfaceState::Down),
            "Unknown" | _ => None,
        };
        Ok(res)
    }
}

pub trait Bridge {
    /*
     * Return all bridges
     */
    fn get_all() -> Result<Vec<impl Bridge>, VirshleError>;
    // fn add_port(&self, iface_type: &InterfaceType) -> Result<impl Interface, VirshleError>;
}

pub trait InterfaceManager {
    fn new() -> Result<impl InterfaceManager, VirshleError>;
    // fn create(iface_type: &InterfaceType) -> Result<impl Interface, VirshleError>;
    // fn delete(iface_name: &str) -> Result<(), VirshleError>;
}

// Manages interfaces with "ovs-vsctl"
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ovs {
    bridges: OvsBridges,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct OvsBridges(Vec<OvsBridge>);

impl InterfaceManager for Ovs {
    fn new() -> Result<Self, VirshleError> {
        let res = Ovs {
            bridges: OvsBridges(OvsBridge::get_all()?),
        };
        Ok(res)
    }
}

// Manages interfaces with "ip"
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ip;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_ovs_manager() -> Result<()> {
        let res = Ovs::new();
        println!("{:#?}", res);
        Ok(())
    }
}
