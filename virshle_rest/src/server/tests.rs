use crate::server::Server;

use virshle_core::{
    config::Config,
    hypervisor::{Vm, VmTable},
    peer::{NodeInfo, Peer},
};
use virshle_network::connection::ConnectionState;

use indexmap::IndexMap;
use pipelight_exec::Status;

// Error Handling
use miette::{Error, Result};
use tracing::{error, info, trace};
use virshle_core::utils::testing;
use virshle_error::{LibError, VirshleError, WrapError};

fn server() -> Result<Server, VirshleError> {
    let config = Config::get()?;
    let server = Server::new().config(&config).build()?;
    Ok(server)
}

// Node methods
#[tokio::test]
async fn node_methods() -> Result<()> {
    testing::tracer()
        .verbosity(tracing::Level::TRACE)
        .db(false)
        .set()?;
    let server = server()?;

    // Ping
    server.api()?.node().ping().await?;

    // Did
    let res: String = server.api()?.node().did().await?;
    println!("\n{:#?}", res);

    // Info
    // Get info for node "Self".
    let res: NodeInfo = server.api()?.node().info().await?;
    println!("\n{:#?}", res);

    // Print node info as a table.
    let node: Peer = server.config.node.into();
    let printable: (Peer, (ConnectionState, Option<NodeInfo>)) =
        (node, (ConnectionState::DaemonUp, Some(res)));
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
    let server = server()?;

    let res: Vec<VmTable> = server.api()?.vm().get().many().exec().await?;
    println!("\n{:#?}", res);

    // Print vms info as a table.
    let peer: Peer = server.config.node.into();
    let printable: IndexMap<Peer, Vec<VmTable>> = IndexMap::from([(peer, res)]);

    testing::logger().verbosity(tracing::Level::WARN).set()?;
    VmTable::display_by_peer(&printable).await?;

    Ok(())
}

// #[tokio::test]
/// Will fail because of nested tokio.
async fn test_http_rest_server() -> Result<()> {
    let server = server()?;
    Ok(())
}
