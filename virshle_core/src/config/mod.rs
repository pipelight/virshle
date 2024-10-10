pub mod cache;
pub mod uri;

// Global vars
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

// Config
use config::Config;
use serde::{Deserialize, Serialize};

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub const MANAGED_DIR: &'static str = "/var/lib/virshle";
pub const CONFIG_DIR: &'static str = "/etc/virshle";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VirshleConfig {
    pub connect: Vec<Node>,
}
impl Default for VirshleConfig {
    fn default() -> Self {
        Self {
            connect: vec![Node::default()],
        }
    }
}
impl VirshleConfig {
    fn get() -> Result<Self, VirshleError> {
        info!("Search config file.");
        Self::get_file("/etc/virshle/config.toml")
    }
    fn get_file(path: &str) -> Result<Self, VirshleError> {
        let settings = Config::builder()
            .add_source(config::File::with_name(path))
            .build()?;
        let config = settings.try_deserialize::<VirshleConfig>()?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Node {
    pub name: String,
    pub url: String,
}
impl Default for Node {
    fn default() -> Self {
        let url = "file://".to_owned() + MANAGED_DIR + "/virshle.sock";
        Self {
            name: "default".to_owned(),
            url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn get_file() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("./virshle.config.toml");
        let path = path.display().to_string();

        let config = VirshleConfig::get()?;
        println!("{:#?}", config);
        Ok(())
    }
}
