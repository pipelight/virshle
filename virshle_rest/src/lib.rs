// Virshle daemon http Rest API
mod client;
mod commons;
mod server;

use virshle_core::config::MANAGED_DIR;

// Socket
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tokio::net::UnixListener;

pub use client::Client;
pub use server::{RestServer, Server};

// Error handling
use miette::Result;
use virshle_error::VirshleError;

impl Server {
    /// Return the virshle daemon default socket path.
    pub fn get_socket() -> Result<String, VirshleError> {
        let path = format!("{MANAGED_DIR}/virshle.sock");
        Ok(path)
    }
    /// Create a unix socket with custom permissions.
    pub async fn make_socket(path: &str) -> Result<UnixListener, VirshleError> {
        let path = PathBuf::from(path);

        // Remove old socket.
        let _ = tokio::fs::remove_file(&path).await;
        tokio::fs::create_dir_all(path.parent().unwrap())
            .await
            .unwrap();

        // Create new socket.
        let listener = UnixListener::bind(&path)?;

        // Set permissions
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o774);
        fs::set_permissions(&path, perms)?;

        Ok(listener)
    }
    pub fn get_host() -> Result<(), VirshleError> {
        Ok(())
    }
}
