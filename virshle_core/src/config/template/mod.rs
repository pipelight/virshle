pub mod disk;
pub mod vm;

use crate::config::VmTemplate;
use crate::hypervisor::Vm;

use bon::bon;
use serde::{Deserialize, Serialize};
use std::fs;

// Error Handling
use miette::Result;
use virshle_error::{CastError, TomlError, VirshleError};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct TemplateConfig {
    pub vm: Option<Vec<VmTemplate>>,
}
#[bon]
impl TemplateConfig {
    #[builder]
    pub fn new(vm: Option<VmTemplate>) -> Self {
        let mut vms = vec![];
        if let Some(vm) = vm {
            vms.push(vm);
        }
        Self { vm: Some(vms) }
    }
}

impl TemplateConfig {
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
            for template in vms {
                let mut vm: Vm = template.try_into()?;
                vm.create(None).await?;
            }
        }
        Ok(self.to_owned())
    }
}
