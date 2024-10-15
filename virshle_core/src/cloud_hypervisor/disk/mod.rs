use vmm::vm_config::DiskConfig as ChDiskConfig;

use serde::{Deserialize, Serialize};

// Error Handling
use miette::{IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};
use virshle_error::{LibError, VirshleError};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct DiskTemplate {
    pub path: String,
    pub readonly: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Disk {
    pub path: String,
    pub readonly: bool,
}
impl From<&DiskTemplate> for Disk {
    fn from(e: &DiskTemplate) -> Self {
        Self {
            path: e.path.to_owned(),
            readonly: match e.readonly {
                Some(x) => x,
                None => false,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    // Error Handling
    use miette::{IntoDiagnostic, Result};

    #[test]
    fn make_handled_disk() -> Result<()> {
        Ok(())
    }
}
