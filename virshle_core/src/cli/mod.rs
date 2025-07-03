mod types;
pub use types::*;

use crate::cloud_hypervisor::VmConfigPlus;
use crate::display::VmTable;
use crate::{
    cloud_hypervisor::{Definition, UserData, Vm, VmState, VmTemplate},
    config::{Node, VirshleConfig},
};
use std::str::FromStr;
use uuid::Uuid;

use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Rest API client
use crate::api::method::vm::{GetManyVmArgs, GetVmArgs};
use crate::api::{rest::client, rest::method, NodeServer};

// Logger
use env_logger::Builder;
use log::LevelFilter;

// Error Handling
use miette::{IntoDiagnostic, Result};

impl Cli {
    pub async fn run() -> Result<()> {
        let cli = Cli::parse();
        Self::switch(cli).await?;
        Ok(())
    }
    pub async fn switch(cli: Cli) -> Result<()> {
        // Set verbosity
        let verbosity = cli.verbose.log_level_filter();
        // Disable sql logs
        let value = format!(
            "{},{}",
            verbosity.to_string().to_lowercase(),
            "sqlx=error,russh=error"
        );
        std::env::set_var("VIRSHLE_LOG", value);
        Builder::from_env("VIRSHLE_LOG").init();

        match cli.commands {
            /*
             * Create the required virshle working directories.
             * Add network ovs entries.
             * Activate network interfaces.
             */
            Commands::Init => {
                VirshleConfig::init().await?;
            }
            Commands::Daemon => {}
            /*
             * Operations on local and remote node
             */
            Commands::Node(args) => match args {
                NodeArgs::Ls => {
                    let res = client::node::get_info().await?;
                    Node::display(res).await?;
                }
                NodeArgs::Ping(ping_args) => {
                    let res = client::node::ping(ping_args.name).await?;
                }
                /*
                 * Serve the node rest API and wait for http requests.
                 */
                NodeArgs::Serve => {
                    NodeServer::run().await?;
                }
            },
            /*
             * Operations on virtual machine templates
             */
            Commands::Template(args) => match args {
                TemplateArgs::Ls => {
                    let res = client::template::get_all().await?;
                    VmTemplate::display_by_nodes(res).await?;
                }
            },
            /*
             * Operations on virtual machines
             */
            Commands::Vm(args) => match args {
                Crud::Create(args) => {
                    if let Some(path) = &args.config {
                        let vm_config_plus = VmConfigPlus::from_file(&path)?;

                        // from cli to api args
                        // client::vm::create(template, None, account).await?;
                    } else {
                        // Create a vm from template.
                        client::vm::create(&args, None).await?;
                    }
                }
                Crud::Info(args) => {
                    if let Some(name) = args.name {
                        let vm = Vm::get_by_name(&name).await?;
                        vm.to_toml()?;
                        vm.get_ch_info().await?;
                    } else if let Some(id) = args.id {
                        let vm = Vm::get_by_id(&id).await?;
                        vm.to_toml()?;
                        vm.get_ch_info().await?;
                    }
                }
                Crud::Start(args) => {
                    let user_data: Option<UserData> = match args.user_data {
                        Some(path) => Some(UserData::from_file(&path)?),
                        None => None,
                    };
                    match args.attach {
                        true => {
                            // Bypass rest API,
                            // and run on local node direcly.
                            method::vm::_start_attach(&args.vm_args, user_data).await?;
                        }
                        _ => {
                            // Rest API
                            client::vm::start(&args.vm_args, user_data).await?;
                        }
                    };
                }
                Crud::Stop(args) => {
                    if args.name.is_some() || args.uuid.is_some() || args.id.is_some() {
                        client::vm::shutdown(
                            GetVmArgs {
                                id: args.id,
                                uuid: args.uuid,
                                name: args.name,
                            },
                            args.node,
                        )
                        .await?;
                    } else if args.state.is_some() || args.account.is_some() {
                        client::vm::shutdown_many(
                            GetManyVmArgs {
                                vm_state: args.state,
                                account_uuid: args.account,
                            },
                            args.node,
                        )
                        .await?;
                    }
                }
                Crud::Ls(args) => {
                    let e: HashMap<Node, Vec<Vm>> = client::vm::get_all(None, None, None).await?;
                    let mut table: HashMap<Node, Vec<VmTable>> = HashMap::new();
                    for (k, v) in e {
                        let vm_table: Vec<VmTable> = VmTable::from_vec(&v).await?;
                        table.insert(k, vm_table);
                    }

                    VmTable::display_by_nodes(table).await?;
                }
                Crud::Rm(args) => {
                    client::vm::delete(&args).await?;
                }
                _ => {}
            },
            // _ => {}
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;
    use miette::Result;

    #[tokio::test]
    async fn parse_command_line() -> Result<()> {
        println!("");
        let e = "virshle --help";
        let os_str: Vec<&str> = e.split(' ').collect();
        let cli = Cli::parse_from(os_str);
        Cli::switch(cli).await?;
        Ok(())
    }

    #[tokio::test]
    async fn get_vms() -> Result<()> {
        println!("");
        let e = "virshle vm ls";
        let os_str: Vec<&str> = e.split(' ').collect();
        let cli = Cli::parse_from(os_str);
        Cli::switch(cli).await?;
        Ok(())
    }
}
