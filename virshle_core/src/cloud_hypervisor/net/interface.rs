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

impl Ip {
    pub fn get_default_interface_name() -> Result<String, VirshleError> {
        let cmd = format!("ip -j route show to default");
        let mut proc = Process::new(&cmd);
        proc.run_piped()?;
        let mut default_routes: Vec<Route> = serde_json::from_str(&proc.io.stdout.unwrap())?;
        let default = default_routes.remove(0);
        Ok(default.dev)
    }
}
