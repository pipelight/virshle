use super::{OvsBridge, OvsInterfaceType, OvsPort};
// Error handling
use log::{error, info};
use miette::{IntoDiagnostic, Result};
use pipelight_exec::Process;
use virshle_error::{LibError, VirshleError, WrapError};

impl OvsBridge {
    pub fn get_port(&self, name: &str) -> Result<OvsPort, VirshleError> {
        let ports: Vec<OvsPort> = self
            .ports
            .iter()
            .filter(|e| e.name == name)
            .map(|e| e.to_owned())
            .collect();
        match ports.first() {
            Some(v) => Ok(v.to_owned()),
            None => {
                let message = format!("Couldn't find port {} on bridge {}", name, self.name);
                let help = format!("Does the port exist?");
                Err(LibError::builder().msg(&message).help(&help).build().into())
            }
        }
    }
    pub fn get_ports_by_type(&self, _type: &OvsInterfaceType) -> Vec<OvsPort> {
        let ports: Vec<OvsPort> = self
            .ports
            .iter()
            .filter(|e| e.interface._type == Some(_type.to_owned()))
            .map(|e| e.to_owned())
            .collect();
        ports
    }
}
