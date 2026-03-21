mod methods;
mod routes;

#[cfg(test)]
mod tests;

// Global vars
use std::sync::{Arc, RwLock};
use virshle_core::config::init::MANAGED_DIR;

// Socket
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tokio::net::UnixListener;

use bon::bon;
use virshle_core::config::Config;

// Error Handling
use miette::Result;
use tracing::info;
use virshle_error::VirshleError;

#[derive(Clone)]
pub struct Server {
    config: Config,
    router: axum::Router,
}

#[bon]
impl Server {
    #[builder(
        start_fn = new,
        finish_fn = build
    )]
    pub fn _new(config: &Config) -> Result<Server, VirshleError> {
        let server = Server {
            config: config.clone(),
            router: axum::Router::default(),
        };
        Ok(server)
    }
}

impl Server {
    /// Run REST api.
    pub async fn serve(&mut self) -> Result<(), VirshleError> {
        let socket_path = Server::get_socket()?;
        self.make_router().await?;

        info!("Server listening on socket {}", &socket_path);
        tokio_scoped::scope(|s| {
            s.spawn(async {
                let listener = Server::make_socket(&socket_path).await.unwrap();
                let _ = axum::serve(listener, self.router.clone()).await;
            });
        });
        Ok(())
    }
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
