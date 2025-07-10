use super::Node;

// Error Handling
use log::{info, warn};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

impl Node {
    pub fn is_best(name: &str, url: &str) -> Result<Self, VirshleError> {
        // Order by weight

        // Order by space left for Vm creation.
        let e = Node {
            name: name.to_owned(),
            url: url.to_owned(),
            weight: 0,
        };

        // Select random in purged list

        Ok(e)
    }
    pub fn is_saturated(&self) -> Result<bool, VirshleError> {
        Ok(false)
    }
}
