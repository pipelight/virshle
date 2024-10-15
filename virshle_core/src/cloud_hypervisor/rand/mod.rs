use bat::PrettyPrinter;
use crossterm::{execute, style::Stylize, terminal::size};
use log::{info, log_enabled, Level};
use rand::seq::SliceRandom;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::Path;

use crate::config::MANAGED_DIR;

// Error Handling
use miette::{IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};
use virshle_error::{LibError, VirshleError};

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
