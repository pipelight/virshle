use super::{Vm, VmTemplate};
use bon::{bon, Builder};
use serde::{Deserialize, Serialize};
use std::fs;

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use virshle_error::{CastError, TomlError, VirshleError, WrapError};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Definition {
    pub vm: Option<Vec<Vm>>,
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
                let err = WrapError::builder()
                    .msg("Couldn't convert definitions string to valid resources")
                    .help("")
                    .origin(err.into())
                    .build();
                return Err(err.into());
            }
        };
        Ok(item)
    }
    // Create
    pub async fn create_all(&mut self) -> Result<Self, VirshleError> {
        self.create_vms().await?;
        Ok(self.to_owned())
    }
    pub async fn create_vms(&mut self) -> Result<Self, VirshleError> {
        if let Some(vms) = &mut self.vm {
            for def in vms {
                def.create().await?;
            }
        }
        Ok(self.to_owned())
    }
    // Delete
    pub async fn delete_all(&self) -> Result<Self, VirshleError> {
        self.delete_vms().await?;
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
    // Start
    pub async fn start_all(&mut self) -> Result<Self, VirshleError> {
        self.start_vms().await?;
        Ok(self.to_owned())
    }
    pub async fn start_vms(&mut self) -> Result<Self, VirshleError> {
        if let Some(vms) = &mut self.vm {
            for def in vms {
                def.start(None, None).await?;
            }
        }
        Ok(self.to_owned())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Template {
    pub vm: Option<Vec<VmTemplate>>,
}
#[bon]
impl Template {
    #[builder]
    pub fn new(vm: Option<VmTemplate>) -> Self {
        let mut vms = vec![];
        if let Some(vm) = vm {
            vms.push(vm);
        }
        Template { vm: Some(vms) }
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

        let mut def = Definition::from_file(&path)?;
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

        let mut def = Definition::from_toml(&toml)?;
        def.create_all().await?;
        Ok(())
    }
}
