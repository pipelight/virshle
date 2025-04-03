use std::fs;
use std::path::Path;
// Error Handling
use miette::{IntoDiagnostic, Result};

use virshle_error::VirshleError;

pub fn ensure_cache_dir() -> Result<(), VirshleError> {
    Path::new("~/.local/share/virshle");
    Ok(())
}

/**
* Convert string into url and fetch ressources to cache.
*/
pub fn get_image_from_url(url: &str) -> Result<(), VirshleError> {
    Ok(())
}

/**
* Check if url ressource is in cache.
*/
pub fn is_cached(url: &str) -> Result<(), VirshleError> {
    Ok(())
}
