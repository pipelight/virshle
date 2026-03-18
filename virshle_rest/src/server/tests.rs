use crate::server::{RestServer, Server};

use virshle_core::{
    config::Config,
    hypervisor::{Vm, VmTable},
    peer::{NodeInfo, Peer},
};
use virshle_network::connection::ConnectionState;

use pipelight_exec::Status;
use std::collections::HashMap;

// Error Handling
use miette::{Error, Result};
use tracing::{error, info, trace};
use virshle_core::utils::testing;
use virshle_error::{LibError, VirshleError, WrapError};

// #[traced_test]
#[tokio::test]
async fn get_node_info() -> Result<()> {
    testing::tracer()
        .verbosity(tracing::Level::TRACE)
        .db(true)
        .set()?;

    // Get Self info.
    let server = Server::new().build()?;
    let res: NodeInfo = server.api()?.node().info().await?;
    info!("{:#?}", res);

    // Print peer info as a table.
    let peer: Peer = Config::get()?.node()?.into();
    let printable = (peer, (ConnectionState::DaemonUp, Some(res)));
    testing::logger().verbosity(tracing::Level::INFO).set()?;
    Peer::display(&printable).await?;

    Ok(())
}

#[tokio::test]
async fn get_vms() -> Result<()> {
    testing::tracer()
        .verbosity(tracing::Level::TRACE)
        .db(true)
        .set()?;

    let server = Server::new().build()?;
    let res: Vec<VmTable> = server.api()?.vm().get().many().exec().await?;
    info!("{:#?}", res);

    // Print vms info as a table.
    let peer: Peer = server.config.node()?.into();
    let printable = HashMap::from([(peer, res)]);

    testing::logger().verbosity(tracing::Level::WARN).set()?;
    VmTable::display_by_peer(&printable).await?;

    Ok(())
}

#[tokio::test]
async fn test_http_rest_server() -> Result<()> {
    RestServer::build().await?.serve().await?;
    Ok(())
}
