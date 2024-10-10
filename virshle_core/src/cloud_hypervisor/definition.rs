use super::vm::Vm;
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
    pub vm: Vec<Vm>,
    // net: Vec<Net>,
    // disk: Vec<Disk>,
}
#[bon]
impl Definition {
    #[builder]
    pub fn new(vm: Option<Vm>) -> Self {
        let mut vms: Vec<Vm> = vec![];
        if let Some(vm) = vm {
            vms.push(vm);
        }
        Definition { vm: vms }
    }
}

impl Definition {
    pub fn from_file(file_path: &str) -> Result<Self, VirshleError> {
        let string = fs::read_to_string(file_path)?;
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
    pub async fn create_vms(&mut self) -> Result<&mut Self, VirshleError> {
        for vm in &mut self.vm {
            vm.create().await?;
        }
        Ok(self)
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
        // def.set_vms()?;

        Ok(())
    }
}
