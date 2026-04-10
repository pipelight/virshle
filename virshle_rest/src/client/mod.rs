#[cfg(test)]
mod tests;

mod display;
mod methods;

use bon::bon;
use indexmap::IndexMap;
use virshle_core::peer::Peer;

// Error Handling
use miette::Result;
use tracing::info;
use virshle_error::VirshleError;

#[derive(Clone)]
pub struct Client {
    peers: IndexMap<String, Peer>,
}

#[bon]
impl Client {
    #[builder(
        start_fn = new,
        finish_fn = build
    )]
    pub fn _new(peers: IndexMap<String, Peer>) -> Result<Client, VirshleError> {
        let client = Client { peers };
        Ok(client)
    }
}
