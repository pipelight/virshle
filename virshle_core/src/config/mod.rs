pub mod cache;
pub mod uri;

use crate::cloud_hypervisor::{NetTemplate, Template, VmTemplate};

use std::collections::HashMap;

// Global vars
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

// Config
use config::Config;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};
use virshle_error::{LibError, VirshleError, WrapError};

pub const MANAGED_DIR: &'static str = "/var/lib/virshle";
pub const CONFIG_DIR: &'static str = "/etc/virshle";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VirshleConfig {
    pub connect: Option<Vec<Node>>,
    pub template: Option<Template>,
}
impl Default for VirshleConfig {
    fn default() -> Self {
        Self {
            connect: Some(vec![Node::default()]),
            template: None,
        }
    }
}
impl VirshleConfig {
    fn debug_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("./virshle.config.toml");
        return path;
    }
    fn release_path() -> PathBuf {
        let mut path = PathBuf::new();
        path.push("/etc/virshle/config.toml");
        return path;
    }
    pub fn get() -> Result<Self, VirshleError> {
        info!("Search config file.");

        #[cfg(debug_assertions)]
        let path = Self::debug_path();

        #[cfg(not(debug_assertions))]
        let path = Self::release_path();

        let path = path.display().to_string();
        let config = Self::from_file(&path)?;
        Ok(config)
    }
    pub fn get_vm_template(&self) -> Result<HashMap<String, VmTemplate>, VirshleError> {
        let mut hashmap = HashMap::new();
        if let Some(template) = &self.template {
            if let Some(vm) = &template.vm {
                hashmap = vm.iter().map(|e| (e.name.clone(), e.to_owned())).collect();
            }
        }
        Ok(hashmap)
    }
    pub fn from_file(path: &str) -> Result<Self, VirshleError> {
        let settings = Config::builder()
            .add_source(config::File::with_name(path))
            .build()?;
        let config = settings.try_deserialize::<VirshleConfig>()?;
        Ok(config)
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

    #[test]
    fn get_config_from_file() -> Result<()> {
        let res = VirshleConfig::get()?;
        // println!("{:#?}", res);
        Ok(())
    }

    #[test]
    fn get_config_from_toml() -> Result<()> {
        let toml = r#"
            [[connect]]
            name = "default"
            url = "file:///var/lib/virshle/virshle.sock"

            [[connect]]
            name = "default-ssh"
            url = "ssh://anon@localhost:22/var/lib/virshle/virshle.sock"

            [template]

            # Vms
            # Standard sizes with decents presets.

            [[template.vm]]
            name = "xs"
            vcpu = 1
            vram = 2
            [[template.vm.disk]]
            path = "~/Iso/nixos.qcow2"
            size = "50G"

            [[template.vm]]
            name = "s"
            vcpu = 2
            vram = 4
            [[template.vm.disk]]
            path = "~/Iso/nixos.qcow2"
            size = "80G"

            [[template.vm]]
            name = "m"
            vcpu = 4
            vram = 8
            size = "100G"

            [[template.vm]]
            name = "l"
            vcpu = 6
            vram = 10
            size = "150G"

            [[template.vm]]
            name = "xl"
            vcpu = 8
            vram = 16
            size = "180G"

            # Default network ipv4 only
            [[template.net]]
            name = "default"
            ip = "192.168.200.1/24"
        "#;

        let res = VirshleConfig::from_toml(&toml)?;
        println!("{:#?}", res);
        Ok(())
    }
}
