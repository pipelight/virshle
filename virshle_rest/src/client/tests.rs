use crate::Client;
use virshle_core::{hypervisor::VmTable, peer::Peer, Vm};

use pipelight_exec::Status;
use std::collections::HashMap;

// Error Handling
use miette::{Error, Result};
use tracing::{error, trace};
use virshle_core::utils::testing;
use virshle_error::{LibError, VirshleError, WrapError};

// #[traced_test]
#[tokio::test]
async fn test_node_client() -> Result<()> {
    testing::tracer()
        .verbosity(tracing::Level::TRACE)
        .db(true)
        .set()?;

    let mut client = Client::api().await?;

    let _res: Result<HashMap<Peer, Vec<VmTable>>, VirshleError> =
        client.vm().get().many().exec().await;

    let _res: Result<VmTable, VirshleError> = client
        .vm()
        .create()
        .one()
        .template("xs")
        .alias("Self")
        .exec()
        .await;
    let _res: Result<Vec<VmTable>, VirshleError> = client
        .vm()
        .create()
        .many()
        .template("xs")
        .alias("Self")
        .n(1)
        .exec()
        .await;

    Ok(())
}
