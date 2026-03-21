#[cfg(test)]
mod tests;

mod display;
mod methods;

use bon::bon;
use virshle_core::config::Config;

// Error Handling
use miette::Result;
use tracing::info;
use virshle_error::VirshleError;

#[derive(Clone)]
pub struct Client {
    config: Config,
}

#[bon]
impl Client {
    #[builder(
        start_fn = new,
        finish_fn = build
    )]
    pub fn _new(config: &Config) -> Result<Client, VirshleError> {
        let client = Client {
            config: config.clone(),
        };
        Ok(client)
    }
}
