use std::fs;
use std::path::Path;
// Random
use rand::prelude::IndexedRandom;

// Error Handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError};

pub fn random_place() -> Result<String, VirshleError> {
    let file = include_str!("places.md");
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
        firstnames.choose(&mut rand::rng()).unwrap(),
        lastnames.choose(&mut rand::rng()).unwrap()
    );

    Ok(res)
}
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
        "{}-{}",
        firstnames.choose(&mut rand::rng()).unwrap(),
        lastnames.choose(&mut rand::rng()).unwrap()
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
        for x in 0..5 {
            let res = random_name()?;
            println!("{}", res);
        }
        Ok(())
    }

    #[test]
    fn gen_random_places() -> Result<()> {
        for x in 0..5 {
            let res = random_place()?;
            println!("{}", res);
        }
        Ok(())
    }
}
