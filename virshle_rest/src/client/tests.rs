use crate::Client;
use virshle_core::{
    config::{Config, UserData},
    hypervisor::{Vm, VmTable},
    peer::{NodeInfo, Peer},
};
use virshle_network::connection::ConnectionState;

use indexmap::IndexMap;
use pipelight_exec::Status;
use std::collections::HashMap;

// Error Handling
use miette::{Error, Result};
use tracing::{debug, error, info, trace};
use virshle_core::utils::testing;
use virshle_error::{LibError, VirshleError, WrapError};

fn client() -> Result<Client, VirshleError> {
    let config = Config::get()?;
    let client = Client::new().config(&config).build()?;
    Ok(client)
}

// Node methods
#[tokio::test]
async fn node_methods() -> Result<()> {
    Ok(())
}
// Peers methods
#[tokio::test]
async fn get_peers() -> Result<()> {
    testing::tracer()
        .verbosity(tracing::Level::TRACE)
        .db(true)
        .set()?;
    let client = client()?;

    let res: IndexMap<Peer, (ConnectionState, Option<NodeInfo>)> =
        client.api().await?.peer().get_info().exec().await?;
    debug!("{:#?}", res);

    testing::logger().verbosity(tracing::Level::WARN).set()?;
    Peer::display_many(res).await?;
    Ok(())
}
#[tokio::test]
async fn get_peers_did() -> Result<()> {
    testing::tracer()
        .verbosity(tracing::Level::TRACE)
        .db(true)
        .set()?;

    let client = client()?;
    let res: IndexMap<Peer, String> = client.api().await?.peer().did().exec().await?;
    debug!("{:#?}", res);
    Ok(())
}

#[tokio::test]
async fn get_vms() -> Result<()> {
    testing::tracer()
        .verbosity(tracing::Level::TRACE)
        .db(true)
        .set()?;

    let client = client()?;
    let res: IndexMap<Peer, Vec<VmTable>> = client.api().await?.vm().get().many().exec().await?;

    testing::logger().verbosity(tracing::Level::WARN).set()?;
    VmTable::display_by_peer(&res).await?;

    Ok(())
}

#[tokio::test]
async fn crud_vm() -> Result<()> {
    testing::tracer()
        .verbosity(tracing::Level::TRACE)
        .db(true)
        .set()?;

    let client = client()?;
    // Create one
    let user_data = UserData::default();
    let vm: VmTable = client
        .api()
        .await?
        .vm()
        .create()
        .one()
        .user_data(user_data)
        .template("xs")
        .alias("Self")
        .exec()
        .await?;
    // Start one
    let _: VmTable = client
        .api()
        .await?
        .vm()
        .start()
        .one()
        .uuid(vm.uuid)
        .alias("Self")
        .exec()
        .await?;
    // Shutdown one
    let _: VmTable = client
        .api()
        .await?
        .vm()
        .shutdown()
        .one()
        .uuid(vm.uuid)
        .alias("Self")
        .exec()
        .await?;
    // Delete one
    let _: VmTable = client
        .api()
        .await?
        .vm()
        .delete()
        .one()
        .uuid(vm.uuid)
        .alias("Self")
        .exec()
        .await?;
    Ok(())
}
