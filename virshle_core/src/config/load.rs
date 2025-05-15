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
    /*
     * Return configuration from default file path.
     */
    pub fn get() -> Result<Self, VirshleError> {
        #[cfg(debug_assertions)]
        let path = Self::debug_path();

        #[cfg(not(debug_assertions))]
        let path = Self::release_path();

        let path = path.display().to_string();
        let config = Self::from_file(&path)?;

        info!("Found config file.");
        Ok(config)
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

            [node]
            [[node]]
            name = "self"
            url = "anon@localhost:22"

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
