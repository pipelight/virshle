use crate::hypervisor::Disk;

use serde::{Deserialize, Serialize};
use std::convert::Into;
use std::path::Path;

// Error Handling
use miette::Result;
use tracing::error;
use virshle_error::{LibError, VirshleError};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct DiskTemplate {
    pub name: String,
    pub path: String,
    pub readonly: Option<bool>,
}
impl DiskTemplate {
    pub fn get_size(&self) -> Result<u64, VirshleError> {
        let source = Self::shellexpand(&self.path)?;
        let path = Path::new(&source);
        if path.exists() && path.is_file() {
            let metadata = std::fs::metadata(path)?;
            let size = metadata.len();
            Ok(size)
        } else {
            Err(LibError::builder()
                .msg("Counldn't get disk file size.")
                .help("Disk doesn't exist or is unreachable")
                .build()
                .into())
        }
    }
    /// Expand tild "~" in file path.
    pub fn shellexpand(relpath: &str) -> Result<String, VirshleError> {
        let source: String = match relpath.starts_with("~") {
            false => relpath.to_owned(),
            true => relpath.replace("~", dirs::home_dir().unwrap().to_str().unwrap()),
        };

        let path = Path::new(&source);
        if path.exists() {
            Ok(source)
        } else {
            let message = format!("Couldn't find file {:#?} expended to {:#?}.", relpath, path);
            error!("{:#?}", message);
            let err = LibError::builder()
                .msg(&message)
                .help("Are you sure the file exist?")
                .build();
            return Err(err.into());
        }
    }
}

impl Into<Disk> for &DiskTemplate {
    fn into(self) -> Disk {
        Disk {
            name: self.name.to_owned(),
            path: self.path.to_owned(),
            readonly: self.readonly,
        }
    }
}
