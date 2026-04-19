use crate::server::Server;
use std::{path::PathBuf, str::FromStr};
use virshle_core::{
    config::{Config, UserData},
    hypervisor::VmTable,
    network::ip::IpInterface,
    utils::testing,
};

use pretty_assertions::Comparison;

// Error Handling
use miette::Result;
use tracing::{error, info};
use virshle_error::{LibError, VirshleError, WrapError};

fn server() -> Result<Server, VirshleError> {
    let config = Config::get()?;
    let server = Server::new().config(&config).build()?;
    Ok(server)
}

/// Verify if the machine is provisionned with user-data:
/// The test tries to:
/// - ssh into the machine with default key-pair
#[tokio::test]
async fn test_ssh_connection() -> Result<(), VirshleError> {
    testing::tracer()
        .verbosity(tracing::Level::DEBUG)
        .db(false)
        .set()?;
    let server = server()?;

    // Create vm with testing ssh key.
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../virshle_core/user-data.toml");
    let user_data = UserData::from_file(path.to_str().unwrap())?;
    let vm: VmTable = server
        .api()?
        .vm()
        .create()
        .one()
        .template("xxs-test")
        .user_data(user_data.clone())
        .exec()
        .await?;

    // Start vm
    server
        .api()?
        .vm()
        .start()
        .one()
        .name(&vm.name)
        .user_data(user_data.clone())
        .exec()
        .await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(15000)).await;

    // Add default testing vm ssh key to ssh-agent.
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../virshle_core/keys/user");
    let ssh_agent_res = testing::exec(&format!("ssh-add {}", path.to_str().unwrap()))?;

    // Ssh into vm and query host info.
    let data = testing::ssh()
        .vm_name(&vm.name)
        .cmd("uname -a")
        .exec()
        .await?;

    // Delete testing vm.
    server
        .api()?
        .vm()
        .delete()
        .one()
        .name(vm.name)
        .exec()
        .await?;

    Ok(())
}

/// Verify if the machine is provisionned with user-data:
/// The test tries to:
/// - ssh into the machine with default key-pair
///
/// DEV WARNING:
/// The ssh-agent can be quite buggy, restart it if needed.
///
/// ```sh
/// pkill ssh-agent && ssh-agent
/// ```
#[tokio::test]
async fn test_persistence_of_dhcp_config() -> Result<(), VirshleError> {
    testing::tracer()
        .verbosity(tracing::Level::DEBUG)
        .db(false)
        .set()?;
    let server = server()?;

    // Create vm with testing ssh key.
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../virshle_core/user-data.toml");
    let user_data = UserData::from_file(path.to_str().unwrap())?;
    let vm: VmTable = server
        .api()?
        .vm()
        .create()
        .one()
        .template("xxs-test")
        .user_data(user_data.clone())
        .exec()
        .await?;

    // Start vm
    server
        .api()?
        .vm()
        .start()
        .one()
        .name(&vm.name)
        .user_data(user_data.clone())
        .exec()
        .await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(15000)).await;

    // Add default testing vm ssh key to ssh-agent.
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../virshle_core/keys/user");
    testing::exec(&format!("ssh-add {}", path.to_str().unwrap()))?;

    // Ssh into vm.
    let data_a = testing::ssh()
        .vm_name(&vm.name)
        .cmd("ip --detail --json --family inet6 address")
        .exec()
        .await?;
    let json_a = serde_json::Value::from_str(&data_a)?;
    println!("{:#?}", json_a);

    let interfaces_a: Vec<IpInterface> = serde_json::from_str(&data_a)?;

    // Fresh start vm
    server
        .api()?
        .vm()
        .start()
        .one()
        .fresh(true)
        .name(&vm.name)
        .user_data(user_data)
        .exec()
        .await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(15000)).await;

    // Ssh into vm.
    let data_b = testing::ssh()
        .vm_name(&vm.name)
        .cmd("ip --detail --json --family inet6 address")
        .exec()
        .await?;
    let json_b = serde_json::Value::from_str(&data_b)?;
    println!("{:#?}", json_b);
    let interfaces_b: Vec<IpInterface> = serde_json::from_str(&data_b)?;

    println!("{}", Comparison::new(&interfaces_a, &interfaces_b));

    // Delete testing vm.
    server
        .api()?
        .vm()
        .delete()
        .one()
        .name(vm.name)
        .exec()
        .await?;
    Ok(())
}

#[tokio::test]
async fn test_init_script() -> Result<(), VirshleError> {
    testing::tracer()
        .verbosity(tracing::Level::DEBUG)
        .db(false)
        .set()?;
    let server = server()?;

    // Create vm with testing ssh key.
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../virshle_core/user-data.toml");
    let user_data = UserData::from_file(path.to_str().unwrap())?;
    let vm: VmTable = server
        .api()?
        .vm()
        .create()
        .one()
        .template("xxs-test")
        .user_data(user_data.clone())
        .exec()
        .await?;

    // Start vm
    server
        .api()?
        .vm()
        .start()
        .one()
        .name(&vm.name)
        .user_data(user_data.clone())
        .exec()
        .await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(15000)).await;

    // Add default testing vm ssh key to ssh-agent.
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../virshle_core/keys/user");
    testing::exec(&format!("ssh-add {}", path.to_str().unwrap()))?;

    // Ssh into vm.
    let data = testing::ssh()
        .vm_name(&vm.name)
        .cmd("systemctl status pipelight-init_net_post.service")
        .exec()
        .await?;
    println!("{:#?}", data);
    let data = testing::ssh()
        .vm_name(&vm.name)
        .cmd("systemctl status pipelight-init_net_pre.service")
        .exec()
        .await?;
    println!("{:#?}", data);

    // Delete testing vm.
    server
        .api()?
        .vm()
        .delete()
        .one()
        .name(vm.name)
        .exec()
        .await?;
    Ok(())
}
