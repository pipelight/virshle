use crate::cloud_hypervisor::{Template, Vm, VmTemplate};
use crate::database;
use crate::network::Ovs;

use super::VirshleConfig;
// Global vars
use super::{CONFIG_DIR, MANAGED_DIR};

// Config
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Error Handling
use log::info;
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{CastError, LibError, TomlError, VirshleError, WrapError};

impl VirshleConfig {
    /*
     * Get config from crate directory
     */
    fn debug_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("./virshle.config.toml");
        return path;
    }
    /*
     * Get config from FHS path.
     */
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
    pub fn get_vm_templates(&self) -> Result<HashMap<String, VmTemplate>, VirshleError> {
        let mut hashmap = HashMap::new();
        if let Some(template) = &self.template {
            if let Some(vm) = &template.vm {
                hashmap = vm.iter().map(|e| (e.name.clone(), e.to_owned())).collect();
            }
        }
        Ok(hashmap)
    }
    pub fn get_template(&self, name: &str) -> Result<VmTemplate, VirshleError> {
        let templates = self.get_vm_templates()?;
        let res = templates.get(name);
        match res {
            Some(res) => Ok(res.to_owned()),
            None => {
                let message = format!("Couldn't find template {:#?}", name);
                let templates_name = templates
                    .iter()
                    .map(|e| e.0.to_owned())
                    .collect::<Vec<String>>()
                    .join(",");
                let help = format!("Available templates are:\n[{templates_name}]");
                let err = LibError::new(&message, &help);
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
        println!("{:#?}", res);
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
            name = "os"
            path = "~/Iso/nixos.efi.raw"
            size = "50G"

            [[template.vm]]
            name = "s"
            vcpu = 2
            vram = 4
            [[template.vm.disk]]
            name = "os"
            path = "~/Iso/nixos.efi.raw"
            size = "80G"

            [[template.vm]]
            name = "m"
            vcpu = 4
            vram = 8
            [[template.vm.disk]]
            name = "os"
            path = "~/Iso/nixos.efi.raw"
            size = "100G"

            [[template.vm]]
            name = "l"
            vcpu = 6
            vram = 10
            [[template.vm.disk]]
            name = "os"
            path = "~/Iso/nixos.efi.raw"
            size = "150G"

            [[template.vm]]
            name = "xl"
            vcpu = 8
            vram = 16
            [[template.vm.disk]]
            name = "os"
            path = "~/Iso/nixos.efi.raw"
            size = "180G"

        "#;

        let res = VirshleConfig::from_toml(&toml)?;
        println!("{:#?}", res);
        Ok(())
    }
}
