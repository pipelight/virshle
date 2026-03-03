use std::collections::HashMap;
use virshle_core::{node::Peer, Vm};

use super::Client;

// Error Handling
use miette::{Error, Result};
use tracing::{error, trace};
use virshle_error::{LibError, VirshleError, WrapError};

#[tokio::test]
async fn test_node_client() -> Result<()> {
    let client = Client::default().api().await?;

    let vms: HashMap<Peer, Vec<Vm>> = client.vm().get().many().exec().await?;

    client.vm().create().one().exec().await?;
    Ok(())
}
