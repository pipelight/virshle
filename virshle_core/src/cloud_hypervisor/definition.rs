use super::{Net, NetTemplate, Vm, VmTemplate};
use serde::{Deserialize, Serialize};
use std::fs;

// Error Handling
use bon::{bon, Builder};
use log::info;
use miette::{IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};
use virshle_error::VirshleError;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Definition {
    pub vm: Vec<VmTemplate>,
    pub net: Vec<NetTemplate>,
}
#[bon]
impl Definition {
    #[builder]
    pub fn new(vm: Option<VmTemplate>, net: Option<NetTemplate>) -> Self {
        let mut vms = vec![];
        if let Some(vm) = vm {
            vms.push(vm);
        }
        let mut nets = vec![];
        if let Some(net) = net {
            nets.push(net);
        }
        Definition { vm: vms, net: nets }
    }
}

impl Definition {
    pub fn from_file(file_path: &str) -> Result<Self, VirshleError> {
        let string = fs::read_to_string(file_path)?;
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
    pub async fn create_all(&self) -> Result<Self, VirshleError> {
        self.create_networks().await?;
        self.create_vms().await?;
        Ok(self.to_owned())
    }
    pub async fn create_vms(&self) -> Result<Self, VirshleError> {
        for def in &self.vm {
            Vm::from(def).create().await?;
        }
        Ok(self.to_owned())
    }
    pub async fn create_networks(&self) -> Result<Self, VirshleError> {
        for def in &self.net {
            Net::from(def).create().await?;
        }
        Ok(self.to_owned())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn set_vm_from_file() -> Result<()> {
        // Get file
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../templates/ch/vm/xs.toml");
        let path = path.display().to_string();

        let toml = r#"
            [[vm]]
            name = "default_xs"
            vcpu = 1
            vram = 2

            [[vm.net]]
            [vm.net.tap]
            name = "default_tap"

            [[net]]
            name = "default_tap"
            ip = "192.168.200.1/24"
        "#;

        let def = Definition::from_toml(&toml)?;
        def.create_all().await?;
        Ok(())
    }
}
