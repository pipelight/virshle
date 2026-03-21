use crate::config::{DhcpType, NodeConfig, Peer, TemplateConfig, UserData};
use crate::hypervisor::VmConfigPlus;
use crate::VmTemplate;

use super::Config;

// Global vars
use once_cell::sync::Lazy;
use std::sync::{Arc, RwLock};

// Config
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// Error Handling
use miette::{Error, Result};
use tracing::{error, trace};
use virshle_error::{CastError, TomlError, VirshleError, WrapError};

pub const CONFIG: Lazy<Arc<RwLock<Option<Config>>>> = Lazy::new(|| Arc::new(RwLock::new(None)));

/// Virshle configuration file structure.
/// This struct is used for deserialization only.
/// It then needs to be converted into a Config struct for in code usage.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PreConfig {
    // Server
    /// The optional local node configuration
    node: Option<NodeConfig>,
    /// Vm templates
    pub template: Option<TemplateConfig>,
    /// Network configuration
    pub dhcp: Option<DhcpType>,
    // Client
    /// List of remote node
    peer: Option<Vec<Peer>>,
}

impl TryInto<Config> for PreConfig {
    type Error = VirshleError;
    fn try_into(self) -> Result<Config, Self::Error> {
        (&self).try_into()
    }
}
impl TryInto<Config> for &PreConfig {
    type Error = VirshleError;
    #[tracing::instrument(skip_all)]
    fn try_into(self) -> Result<Config, Self::Error> {
        let mut config = Config {
            dhcp: self.dhcp.clone(),
            ..Config::default()
        };
        // Node conversion
        if let Some(node) = &self.node {
            config.node = node.try_into()?;
        }
        // Template conversion
        if let Some(templates) = &self.template {
            if let Some(vm_templates) = &templates.vm {
                for e in vm_templates {
                    config.templates.insert(e.name.clone(), e.clone());
                }
            }
        }
        // Peer conversion
        if let Some(peer) = &self.peer {
            for e in peer {
                config.peers.insert(e.alias.clone(), e.clone());
            }
        }

        Ok(config)
    }
}

impl PreConfig {
    /// Get config from crate directory
    #[tracing::instrument(skip_all)]
    fn debug_path() -> Result<PathBuf, io::Error> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("./virshle.config.toml");
        path = path.as_path().canonicalize()?;
        trace!("Reading config file at {:#?}", path);
        Ok(path)
    }
    /// Get config from FHS path.
    #[tracing::instrument(skip_all)]
    fn release_path() -> Result<PathBuf, io::Error> {
        let mut path = PathBuf::new();
        path.push("/etc/virshle/config.toml");
        path = path.as_path().canonicalize()?;
        trace!("Reading config file at {:#?}", path);
        Ok(path)
    }

    /// Return configuration from default file path.
    #[tracing::instrument]
    pub fn get() -> Result<Self, VirshleError> {
        // Early return config if already stored in memory
        // let config = CONFIG.read().unwrap().clone();
        // match config {
        //     Some(v) => return Ok(v),
        //     None => {}
        // };

        #[cfg(debug_assertions)]
        let path = Self::debug_path()?;
        #[cfg(not(debug_assertions))]
        let path = Self::release_path()?;
        let path = path.display().to_string();
        match Self::from_file(&path) {
            Ok(v) => {
                trace!("Loaded config file.");
                // *CONFIG.write().unwrap() = Some(v.clone());
                Ok(v)
            }
            Err(e) => {
                let message = format!("Couldn't find a configuration file.",);
                let help = format!("Create a configuration file at {:#?}", path);
                let err = WrapError::builder()
                    .msg(&message)
                    .help(&help)
                    .origin(Error::from_err(e))
                    .build();

                error!("{}", err);
                Err(err.into())
            }
        }
    }
    pub fn from_file(path: &str) -> Result<Self, VirshleError> {
        let string = fs::read_to_string(path)?;
        Self::from_toml(&string)
    }
    pub fn from_toml(string: &str) -> Result<Self, VirshleError> {
        let res = toml::from_str::<Self>(&string);
        let item = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlError(TomlError::new(e, &string));
                return Err(err.into());
            }
        };
        Ok(item)
    }
}

impl UserData {
    pub fn from_file(path: &str) -> Result<Self, VirshleError> {
        let string = fs::read_to_string(path)?;
        Self::from_toml(&string)
    }
    pub fn from_toml(string: &str) -> Result<Self, VirshleError> {
        let res = toml::from_str::<Self>(&string);
        let item = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlError(TomlError::new(e, &string));
                return Err(err.into());
            }
        };
        Ok(item)
    }
}
impl VmConfigPlus {
    pub fn from_file(path: &str) -> Result<Self, VirshleError> {
        let string = fs::read_to_string(path)?;
        Self::from_toml(&string)
    }
    pub fn from_toml(string: &str) -> Result<Self, VirshleError> {
        let res = toml::from_str::<Self>(&string);
        let item = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlError(TomlError::new(e, &string));
                return Err(err.into());
            }
        };
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[test]
    fn get_config_from_file() -> Result<()> {
        // let res = Config::get()?;
        // Test loading from memory
        // Config::get()?;
        // println!("{:#?}", res);
        Ok(())
    }

    #[test]
    fn get_config_from_toml() -> Result<()> {
        let toml = r#"

            [node]
            name = "self"
            url = "anon@localhost:22"
            private_key = "/var/lib/virshle/keys/private_key"
            public_key = "/var/lib/virshle/keys/public_key"

            [[peer]]
            url = "ssh://anon@remote:22/var/lib/virshle/virshle.sock"
            weight = 20


            [template]
            # Vms
            # Standard sizes with decents presets.

            [[template.vm]]
            name = "xs"
            vcpu = 1
            vram = "2GiB"
            [[template.vm.disk]]
            name = "os"
            path = "~/Iso/nixos.efi.raw"
            size = "50G"

            [[template.vm]]
            name = "s"
            vcpu = 2
            vram = "4GiB"
            [[template.vm.disk]]
            name = "os"
            path = "~/Iso/nixos.efi.raw"
            size = "80G"

            [[template.vm]]
            name = "m"
            vcpu = 4
            vram = "8GiB"
            [[template.vm.disk]]
            name = "os"
            path = "~/Iso/nixos.efi.raw"
            size = "100G"

            [[template.vm]]
            name = "l"
            vcpu = 6
            vram = "10GiB"
            [[template.vm.disk]]
            name = "os"
            path = "~/Iso/nixos.efi.raw"
            size = "150G"

            [[template.vm]]
            name = "xl"
            vcpu = 8
            vram = "16GiB"
            [[template.vm.disk]]
            name = "os"
            path = "~/Iso/nixos.efi.raw"
            size = "180G"

        "#;

        let res = PreConfig::from_toml(&toml)?;
        println!("{:#?}", res);
        Ok(())
    }
}
