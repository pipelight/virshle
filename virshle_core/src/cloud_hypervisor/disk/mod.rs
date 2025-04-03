// Struct
use super::Vm;
use std::fs;
use std::path::PathBuf;

// Cloud Hypervisor
use uuid::Uuid;

use serde::{Deserialize, Serialize};

// Error Handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct DiskTemplate {
    pub name: String,
    pub path: String,
    pub readonly: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Disk {
    pub name: String,
    pub path: String,
    pub readonly: Option<bool>,
}
impl From<&DiskTemplate> for Disk {
    fn from(e: &DiskTemplate) -> Self {
        Self {
            name: "os".to_owned(),
            path: e.path.to_owned(),
            readonly: e.readonly,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Error Handling
    use miette::{IntoDiagnostic, Result};

    #[test]
    fn make_handled_disk() -> Result<()> {
        Ok(())
    }
}
