use pipelight_exec::{Process, Status};
use serde::{Deserialize, Serialize};
use std::fs;
use tabled::{Table, Tabled};

// Error Handling
use log::info;
use miette::{Error, IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Ip;

#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Route {
    dst: String,
    gateway: String,
    dev: String,
    protocol: String,
    prefsrc: String,
    metric: u8,
    flags: Vec<String>,
}
#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Link {
    ifindex: u8,
    ifname: String,
    flags: Vec<String>,
    mtu: u8,
    qdisc: String,
    operstate: LinkState,
    linkmode: String,
    group: String,
    txqlen: u8,
    link_type: u8,
    address: String,
    broadcast: String,
    altnames: Vec<String>,
}
#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum LinkState {
    Up,
    Down,
    #[default]
    NotCreated,
}

impl Ip {
    pub fn get_default_interface_name() -> Result<String, VirshleError> {
        let cmd = format!("ip -j route show to default");
        let mut proc = Process::new(&cmd);
        proc.run_piped()?;
        let mut default_routes: Vec<Route> = serde_json::from_str(&proc.io.stdout.unwrap())?;
        let default = default_routes.remove(0);
        Ok(default.dev)
    }
    pub fn get_interface_state(name: &str) -> Result<LinkState, VirshleError> {
        let cmd = format!("ip -j link show {name}");
        let mut proc = Process::new(&cmd);
        proc.run_piped()?;

        match proc.state.status {
            Some(Status::Failed) => Ok(LinkState::NotCreated),
            Some(Status::Succeeded) => {
                let mut links: Vec<Link> = serde_json::from_str(&proc.io.stdout.unwrap())?;
                let link = links.remove(0);
                Ok(link.operstate)
            }
            _ => {
                let message = "Couldn't get network interface state";
                Err(LibError::new(message, "").into())
            }
        }
    }
}
