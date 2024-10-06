use crate::config::MANAGED_DIR;
use std::fs;
use std::path::Path;

// Error Handling
use crate::error::{VirshleError, WrapError};
use log::{info, log_enabled, Level};
use miette::{IntoDiagnostic, Result};

pub fn clean() -> Result<(), VirshleError> {
    let files = fs::read_dir(&Path::new(&*MANAGED_DIR.lock().unwrap()).join("files"))?;
    // Remove files if unused
    for file in files {
        let res = fs::remove_file(file?.path());
        match res {
            Ok(_) => {}
            Err(_) => {}
        };
    }

    Ok(())
}
