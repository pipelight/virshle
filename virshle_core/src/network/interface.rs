use super::ovs;
use super::ovs::{OvsBridge, OvsInterface};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, Value::Array};

// Error handling
use log::{error, info};
use miette::{IntoDiagnostic, Result};
use pipelight_exec::Process;
use virshle_error::{LibError, VirshleError, WrapError};

pub trait Bridge {
    fn get_all() -> Result<Vec<impl Bridge>, VirshleError>;

    fn get_ports<'a>(&mut self) -> Result<&mut Self, VirshleError>
    where
        Self: Bridge + Serialize + Deserialize<'a>;
}

pub trait InterfaceLinker {
    /*
     * Link an interface to a bridge
     */
    fn bridge(br: &str, iface: &str) -> Result<(), VirshleError>;
}

pub trait InterfaceManager {
    fn get_all() -> Result<(), VirshleError>;
    fn create() -> Result<(), VirshleError>;
    fn delete() -> Result<(), VirshleError>;
}

// Manages interfaces with ovs-vsctl
pub struct Ovs;

impl InterfaceManager for Ovs {
    fn get_all() -> Result<(), VirshleError> {
        Ok(())
    }
    fn create() -> Result<(), VirshleError> {
        Ok(())
    }
    fn delete() -> Result<(), VirshleError> {
        Ok(())
    }
}

// Manages interfaces with ip
pub struct Ip;

impl InterfaceManager for Ip {
    fn get_all() -> Result<(), VirshleError> {
        Ok(())
    }
    fn create() -> Result<(), VirshleError> {
        Ok(())
    }
    fn delete() -> Result<(), VirshleError> {
        Ok(())
    }
}
