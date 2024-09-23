use bat::PrettyPrinter;
use crossterm::{execute, style::Stylize, terminal::size};
use log::{info, log_enabled, Level};
use rand::seq::SliceRandom;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;

// Error Handling
use super::toml::make_canonical_path;
use crate::error::LibError;
use crate::error::VirshleError;
use miette::{IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};

pub fn random_name() -> Result<String, VirshleError> {
    let file = include_str!("names.md");
    let names: Vec<String> = file
        .split("\n")
        .filter(|e| !e.starts_with("#"))
        .filter(|&e| e != "")
        .map(|e| e.trim().to_owned())
        .collect();

    let firstnames: Vec<String> = names
        .clone()
        .iter()
        .map(|e| e.split(" ").next().unwrap().to_owned())
        .collect();
    let lastnames: Vec<String> = names
        .clone()
        .iter()
        .map(|e| e.split(" ").last().unwrap().to_owned())
        .collect();

    let res = format!(
        "{}_{}",
        firstnames.choose(&mut rand::thread_rng()).unwrap(),
        lastnames.choose(&mut rand::thread_rng()).unwrap()
    );
    Ok(res)
}

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

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    // Error Handling
    use miette::{IntoDiagnostic, Result};

    #[test]
    fn gen_random_name() -> Result<()> {
        random_name()?;
        Ok(())
    }
}
