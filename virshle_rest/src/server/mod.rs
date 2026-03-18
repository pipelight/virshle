mod methods;
mod routes;
mod tests;

// Global vars
use std::sync::{Arc, RwLock};

use bon::bon;
use virshle_core::config::Config;

// Error Handling
use miette::Result;
use tracing::info;
use virshle_error::VirshleError;

#[derive(Clone)]
pub struct Server {
    config: Config,
}
impl Default for Server {
    fn default() -> Self {
        Server {
            config: Config::default(),
        }
    }
}

#[bon]
impl Server {
    #[builder(start_fn = new ,finish_fn = build)]
    pub fn _new() -> Result<Server, VirshleError> {
        let mut server = Server::default();
        let config = Config::get()?;
        server.config = config;
        Ok(server)
    }
}

#[derive(Clone)]
pub struct RestServer {
    router: axum::Router,
}
impl RestServer {
    /// Run REST api.
    pub async fn serve(&self) -> Result<(), VirshleError> {
        let socket_path = Server::get_socket()?;

        info!("Server listening on socket {}", &socket_path);
        tokio_scoped::scope(|s| {
            s.spawn(async {
                let listener = Server::make_socket(&socket_path).await.unwrap();
                let _ = axum::serve(listener, self.router.clone()).await;
            });
        });
        Ok(())
    }
}
