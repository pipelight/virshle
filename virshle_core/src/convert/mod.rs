pub mod toml;
pub mod xml;

pub use toml::from_toml;
pub use xml::to_xml;

// Error Handling
use crate::error::VirshleError;
use miette::{IntoDiagnostic, Result};

pub fn from_toml_to_xml(toml: &str) -> Result<String, VirshleError> {
    let value = from_toml(toml)?;
    let xml = to_xml(&value)?;
    Ok(xml)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    // Error Handling
    use miette::{IntoDiagnostic, Result};

    #[test]
    fn try_from_toml_to_xml() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../templates/vm/base.toml");
        let path = path.to_str().unwrap();

        let toml = fs::read_to_string(path).into_diagnostic()?;

        println!("\n{}", toml);
        let value = from_toml(&toml)?;

        println!("\n{:#?}", value);
        let xml = to_xml(&value)?;

        println!("\n{}", xml);
        Ok(())
    }
}
