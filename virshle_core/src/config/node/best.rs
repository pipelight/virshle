use super::Node;

// Error Handling
use log::{info, warn};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

impl Node {
    pub fn get_best(name: &str, url: &str) -> Result<Self, VirshleError> {
        let e = Node {
            name: name.to_owned(),
            url: url.to_owned(),
        };
        Ok(e)
    }
}
