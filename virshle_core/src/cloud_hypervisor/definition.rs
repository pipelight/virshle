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
    pub vm: Option<Vec<Vm>>,
    pub net: Option<Vec<Net>>,
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
    // Create
    pub async fn create_all(&self) -> Result<Self, VirshleError> {
        self.create_networks().await?;
        self.create_vms().await?;
        Ok(self.to_owned())
    }
    pub async fn create_vms(&self) -> Result<Self, VirshleError> {
        if let Some(vms) = &self.vm {
            for def in vms {
                def.create().await?;
            }
        }
        Ok(self.to_owned())
    }
    pub async fn create_networks(&self) -> Result<Self, VirshleError> {
        if let Some(nets) = &self.net {
            for def in nets {
                def.create().await?;
            }
        }
        Ok(self.to_owned())
    }
    // Delete
    pub async fn delete_all(&self) -> Result<Self, VirshleError> {
        self.delete_vms().await?;
        self.delete_networks().await?;
        Ok(self.to_owned())
    }
    pub async fn delete_vms(&self) -> Result<Self, VirshleError> {
        if let Some(vms) = &self.vm {
            for def in vms {
                def.delete().await?;
            }
        }
        Ok(self.to_owned())
    }
    pub async fn delete_networks(&self) -> Result<Self, VirshleError> {
        if let Some(nets) = &self.net {
            for def in nets {
                def.delete().await?;
            }
        }
        Ok(self.to_owned())
    }
    // Start
    pub async fn start_all(&self) -> Result<Self, VirshleError> {
        self.start_networks().await?;
        self.start_vms().await?;
        Ok(self.to_owned())
    }
    pub async fn start_vms(&self) -> Result<Self, VirshleError> {
        if let Some(vms) = &self.vm {
            for def in vms {
                def.start().await?;
            }
        }
        Ok(self.to_owned())
    }
    pub async fn start_networks(&self) -> Result<Self, VirshleError> {
        if let Some(nets) = &self.net {
            for def in nets {
                def.start().await?;
            }
        }
        Ok(self.to_owned())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Template {
    pub vm: Option<Vec<VmTemplate>>,
    pub net: Option<Vec<NetTemplate>>,
}
#[bon]
impl Template {
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
        Template {
            vm: Some(vms),
            net: Some(nets),
        }
    }
}

impl Template {
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
        if let Some(vms) = &self.vm {
            for def in vms {
                Vm::from(def).create().await?;
            }
        }
        Ok(self.to_owned())
    }
    pub async fn create_networks(&self) -> Result<Self, VirshleError> {
        if let Some(nets) = &self.net {
            for def in nets {
                Net::from(def).create().await?;
            }
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

        let def = Definition::from_file(&path)?;
        def.create_all().await?;
        Ok(())
    }

    #[tokio::test]
    async fn set_vm_from_toml() -> Result<()> {
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
