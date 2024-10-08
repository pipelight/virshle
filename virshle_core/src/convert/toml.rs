use bat::PrettyPrinter;
use crossterm::{execute, style::Stylize, terminal::size};
use log::{info, log_enabled, Level};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;

use super::path;

// Error Handling
use virshle_error::LibError;
use virshle_error::VirshleError;
use miette::{IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};

/**
* Returns a toml string from a Value
*/
pub fn to_toml(value: &Value) -> Result<String, CastError> {
    let res = toml::to_string(value)?;
    Ok(res)
}
/**
* Returns a Value from a toml string
*/
pub fn from_toml(string: &str) -> Result<Value, VirshleError> {
    let res = toml::from_str::<Value>(string);

    if log_enabled!(Level::Info) {
        let (cols, _) = size()?;
        let divider = "-".repeat((cols / 3).into());
        println!("{}", format!("{divider}toml{divider}").green());
        PrettyPrinter::new()
            .input_from_bytes(string.as_bytes())
            .language("toml")
            .print()?;
        println!("");
    }

    match res {
        Ok(mut res) => {
            path::relpath_to_fullpath(&mut res)?;
            Ok(res)
        }
        Err(e) => {
            let err = CastError::TomlError(TomlError::new(e, &string));
            Err(err.into())
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
    fn read_file_to_string() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../templates/vm/default.toml");
        let path = path.to_str().unwrap();
        let string = fs::read_to_string(path).into_diagnostic()?;

        Ok(())
    }

    #[test]
    fn load_toml_file() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../templates/vm/default.toml");
        let path = path.to_str().unwrap();
        let string = fs::read_to_string(path).into_diagnostic()?;

        println!("");
        let res = from_toml(&string)?;
        println!("{:#?}", res);

        Ok(())
    }
}
