use serde_json::{json, Map, Value};
use std::fs;
// Error Handling
use crate::error::VirshleError;
use miette::{IntoDiagnostic, Result};

use pipelight_utils::files::{CastError, TomlError};

/**
Returns a Value from a toml string
*/
pub fn from_toml(string: &str) -> Result<Value, VirshleError> {
    let res = toml::from_str::<Value>(string);
    match res {
        Ok(res) => Ok(res),
        Err(e) => {
            let err = CastError::TomlError(TomlError::new(e, &string));
            Err(err.into())
        }
    }
}
pub fn relpath_to_fullpath(value: Value) -> Result<Value> {
    Ok(value)
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
        path.push("../templates/vm/base.toml");
        let path = path.to_str().unwrap();
        let string = fs::read_to_string(path).into_diagnostic()?;
        Ok(())
    }

    #[test]
    fn load_toml_file() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../templates/vm/base.toml");
        let path = path.to_str().unwrap();

        let string = fs::read_to_string(path).into_diagnostic()?;

        let res = from_toml(&string)?;
        println!("{:#?}", res);

        Ok(())
    }
}
