use std::path::Path;

// Error Handling
use log::{debug, error, info, trace};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError};

/// Convert string to bytes.
pub fn reverse_human_bytes(string: &str) -> Result<u64, VirshleError> {
    if string.strip_suffix("TiB").is_some() {
        let num: &str = string.trim_end_matches("TiB");
        let int: u64 = num.parse()?;
        let int = int * u64::pow(1024, 4);
        Ok(int)
    } else if string.strip_suffix("GiB").is_some() {
        let num: &str = string.trim_end_matches("GiB");
        let int: u64 = num.parse()?;
        let int = int * u64::pow(1024, 3);
        Ok(int)
    } else if string.strip_suffix("MiB").is_some() {
        let num: &str = string.trim_end_matches("GiB");
        let int: u64 = num.parse()?;
        let int = int * u64::pow(1024, 2);
        Ok(int)
    } else if string.strip_suffix("KiB").is_some() {
        let num: &str = string.trim_end_matches("GiB");
        let int: u64 = num.parse()?;
        let int = int * u64::pow(1024, 1);
        Ok(int)
    } else if string.strip_suffix("B").is_some() {
        let num: &str = string.trim_end_matches("GiB");
        let int: u64 = num.parse()?;
        Ok(int)
    } else {
        Err(LibError::builder()
            .msg("Couldn't convert human readable string to bytes")
            .help("Must be of the form 50GiB, 2MiB, 110KiB or 1B")
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
