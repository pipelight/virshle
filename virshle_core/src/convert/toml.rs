use bat::PrettyPrinter;
use crossterm::{execute, style::Stylize, terminal::size};
use log::{info, log_enabled, Level};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;

// Error Handling
use crate::error::LibError;
use crate::error::VirshleError;
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
            relpath_to_fullpath(&mut res)?;
            Ok(res)
        }
        Err(e) => {
            let err = CastError::TomlError(TomlError::new(e, &string));
            Err(err.into())
        }
    }
}

pub fn relpath_to_fullpath(value: &mut Value) -> Result<(), VirshleError> {
    if let Some(map) = value.as_object_mut() {
        let binding = map.clone();
        let keys = binding.keys();
        for key in keys {
            if let Some(mut v) = map.get_mut(key) {
                make_canonical_path(key, &mut v)?;
            }
        }
    }
    Ok(())
}

pub fn make_canonical_path(key: &str, value: &mut Value) -> Result<(), VirshleError> {
    let tags = ["@file".to_owned(), "#text".to_owned()];
    match value {
        Value::Object(map) => {
            for (k, mut v) in map {
                make_canonical_path(k, &mut v)?;
            }
        }
        Value::Array(value) => {
            for e in value {
                make_canonical_path(key, e)?;
            }
        }
        Value::String(string) => {
            if tags.contains(&key.to_string()) {
                if string.contains("./")
                    || string.contains("../")
                    || string.contains("/../")
                    || string.contains("~/")
                {
                    let string = shellexpand::tilde(string).to_string();
                    let path = Path::new(&string);

                    if path.exists() {
                        let abs_path = path.canonicalize()?;
                        let abs_path = abs_path.display().to_string();
                        *value = Value::String(abs_path);
                    } else {
                        let message = format!("The file at path {:?} doesn't exist.", string);
                        let help = format!("change the path for key {:?}", key);
                        let err = LibError { message, help };
                        return Err(err.into());
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
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
