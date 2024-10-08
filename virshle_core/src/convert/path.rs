use bat::PrettyPrinter;
use crossterm::{execute, style::Stylize, terminal::size};
use log::{info, log_enabled, Level};
use rand::seq::SliceRandom;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;

use crate::config::MANAGED_DIR;

// Error Handling
use virshle_error::LibError;
use virshle_error::VirshleError;
use miette::{IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};

pub fn relpath_to_copy(value: &mut Value, uuid: &str) -> Result<(), VirshleError> {
    if let Some(map) = value.as_object_mut() {
        let binding = map.clone();
        let keys = binding.keys();
        for key in keys {
            if let Some(mut v) = map.get_mut(key) {
                make_managed_path(key, &mut v, uuid)?;
            }
        }
    }
    Ok(())
}
/*
* Copy source files to /var/lib/virshle.
*/
pub fn make_managed_path(key: &str, value: &mut Value, uuid: &str) -> Result<(), VirshleError> {
    let tags = ["@file".to_owned()];
    match value {
        Value::Object(map) => {
            for (k, mut v) in map {
                make_managed_path(k, &mut v, uuid)?;
            }
        }
        Value::Array(value) => {
            for e in value {
                make_managed_path(key, e, uuid)?;
            }
        }
        Value::String(string) => {
            let dir = "/var/lib/virshle/files/";
            fs::create_dir_all(dir)?;

            if tags.contains(&key.to_string()) {
                let origin = Path::new(&string);
                if origin.exists() && !origin.is_dir() {
                    let filename =
                        format!("{}_{}", uuid, origin.file_name().unwrap().to_str().unwrap());
                    let destination = Path::new(dir).join(filename).to_str().unwrap().to_owned();
                    fs::copy(origin, destination.clone())?;
                    *value = Value::String(destination.clone());
                }
            }
        }
        _ => {}
    }
    Ok(())
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
