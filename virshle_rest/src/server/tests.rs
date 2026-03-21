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

// Server initilisation
#[tokio::test]
async fn server() -> Result<()> {
    testing::tracer()
        .verbosity(tracing::Level::TRACE)
        .db(false)
        .set()?;

    let server = Server::new().build()?;
    server.api()?;

    Ok(())
}

// Node methods
#[tokio::test]
async fn node_methods() -> Result<()> {
    testing::tracer()
        .verbosity(tracing::Level::TRACE)
        .db(false)
        .set()?;

    let server = Server::new().build()?;

    // Ping
    server.api()?.node().ping().await?;

    // Did
    let res: String = server.api()?.node().did().await?;
    info!("\n{:#?}", res);

    // Info
    // Get info for node "Self".
    let res: NodeInfo = server.api()?.node().info().await?;
    info!("\n{:#?}", res);
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

// #[tokio::test]
/// Will fail because of nested tokio.
async fn test_http_rest_server() -> Result<()> {
    RestServer::build().await?.serve().await?;
    Ok(())
}
