use crate::Client;
use virshle_core::{
    hypervisor::{Vm, VmTable},
    peer::{NodeInfo, Peer},
};
use virshle_network::connection::ConnectionState;

use pipelight_exec::Status;
use std::collections::HashMap;

// Error Handling
use miette::{Error, Result};
use tracing::{debug, error, info, trace};
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
    // let _res: Result<VmTable, VirshleError> = client
    //     .vm()
    //     .delete()
    //     .one()
    //     .uuid(res.)
    //     .alias("Self")
    //     .exec()
    //     .await;

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

#[tokio::test]
async fn get_peers() -> Result<()> {
    testing::tracer()
        .verbosity(tracing::Level::TRACE)
        .db(true)
        .set()?;

    let mut client = Client::api().await?;
    let res: HashMap<Peer, (ConnectionState, Option<NodeInfo>)> =
        client.peer().get_info().exec().await?;
    debug!("{:#?}", res);

    testing::logger().verbosity(tracing::Level::WARN).set()?;
    Peer::display_many(res).await?;

    Ok(())
}

#[tokio::test]
async fn get_vms() -> Result<()> {
    testing::tracer()
        .verbosity(tracing::Level::TRACE)
        .db(true)
        .set()?;

    let mut client = Client::api().await?;
    let res: HashMap<Peer, Vec<VmTable>> = client.vm().get().many().exec().await?;

    testing::logger().verbosity(tracing::Level::WARN).set()?;
    VmTable::display_by_peer(&res).await?;

    Ok(())
}
