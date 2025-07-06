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

// Spinners
use owo_colors::OwoColorize;
use spinoff::{spinners, Color, Spinner};

// Rest API client
use crate::api::method::vm::{CreateVmArgs, GetManyVmArgs, GetVmArgs};
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

        // Set working node
        let cw_node = cli.current_workgin_node.node;

        match cli.commands {
            /*
             * Create the required virshle working directories.
             * Add network ovs entries.
             * Activate network interfaces.
             */
            Commands::Init => {
                VirshleConfig::init().await?;
            }
            /*
             * Operations on local and remote node
             */
            Commands::Node(args) => match args {
                NodeArgs::Ls => {
                    let res = client::node::get_info_all().await?;
                    Node::display(res).await?;
                }
                NodeArgs::Ping(args) => {
                    let res = client::node::ping(args.node).await?;
                }
                NodeArgs::Info(args) => {
                    let res = client::node::get_info(args.node).await?;
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
                    // Create a vm from template.
                    client::vm::create(
                        CreateVmArgs {
                            template_name: args.template,
                            account_uuid: None,
                        },
                        None,
                    )
                    .await?;
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
                            let args = args.vm_args;
                            // Bypass rest API,
                            // and run on local node direcly.
                            method::vm::_start_attach(
                                GetVmArgs {
                                    id: args.id,
                                    uuid: args.uuid,
                                    name: args.name,
                                },
                                user_data,
                            )
                            .await?;
                        }
                        _ => {
                            let args = args.vm_args;
                            if args.name.is_some() || args.uuid.is_some() || args.id.is_some() {
                                // Spinner
                                let mut sp =
                                    Spinner::new(spinners::Toggle5, "Starting vm...", None);
                                // Rest API
                                let vm = client::vm::start(
                                    GetVmArgs {
                                        id: args.id,
                                        uuid: args.uuid,
                                        name: args.name,
                                    },
                                    user_data,
                                    cw_node.clone(),
                                )
                                .await?;

                                // Spinner
                                let node = Node::unwrap_or_default(cw_node).await?;
                                let vm_name = format!("vm-{}", vm.name);
                                let message = format!(
                                    "Started {} on node {}",
                                    vm_name.bold().blue(),
                                    node.name.bold().green()
                                );
                                sp.stop_and_persist("✅", &message);
                            } else if args.state.is_some() || args.account.is_some() {
                                // Spinner
                                let mut sp =
                                    Spinner::new(spinners::Toggle5, "Starting vms...", None);
                                let vms = client::vm::start_many(
                                    GetManyVmArgs {
                                        vm_state: args.state,
                                        account_uuid: args.account,
                                    },
                                    cw_node.clone(),
                                )
                                .await?;
                                let vms_name: Vec<String> = vms
                                    .iter()
                                    .map(|e| format!("vm-{}", e.name.bold().blue()))
                                    .collect();
                                let vms_name: String = vms_name.join("\n");

                                // Spinner
                                let node = Node::unwrap_or_default(cw_node).await?;
                                let message = format!(
                                    "Started [{}] on node {}",
                                    vms_name,
                                    node.name.bold().green()
                                );
                                sp.stop_and_persist("✅", &message);
                            }
                        }
                    };
                }
                Crud::Stop(args) => {
                    if args.name.is_some() || args.uuid.is_some() || args.id.is_some() {
                        // Spinner
                        let mut sp = Spinner::new(spinners::Toggle5, "Stopping vm...", None);
                        let vm = client::vm::shutdown(
                            GetVmArgs {
                                id: args.id,
                                uuid: args.uuid,
                                name: args.name,
                            },
                            cw_node.clone(),
                        )
                        .await?;

                        // Spinner
                        let node = Node::unwrap_or_default(cw_node).await?;
                        let vm_name = format!("vm-{}", vm.name);
                        let message = format!(
                            "Stopped {} on node {}",
                            vm_name.bold().blue(),
                            node.name.bold().green()
                        );
                        sp.stop_and_persist("✅", &message);
                    } else if args.state.is_some() || args.account.is_some() {
                        // Spinner
                        let mut sp = Spinner::new(spinners::Toggle5, "Stopping vms...", None);
                        let vms = client::vm::shutdown_many(
                            GetManyVmArgs {
                                vm_state: args.state,
                                account_uuid: args.account,
                            },
                            cw_node.clone(),
                        )
                        .await?;
                        let vms_name: Vec<String> = vms
                            .iter()
                            .map(|e| format!("vm-{}", e.name.bold().blue()))
                            .collect();
                        let vms_name: String = vms_name.join("\n");

                        // Spinner
                        let node = Node::unwrap_or_default(cw_node).await?;
                        let message = format!(
                            "Stopped [{}] on node {}",
                            vms_name,
                            node.name.bold().green()
                        );
                        sp.stop_and_persist("✅", &message);
                    }
                }
                Crud::Ls(args) => {
                    let table = client::vm::get_info_many(
                        Some(GetManyVmArgs {
                            vm_state: args.state,
                            account_uuid: args.account,
                        }),
                        cw_node,
                    )
                    .await?;
                    VmTable::display_by_nodes(table).await?;
                }
                Crud::Rm(args) => {
                    if args.name.is_some() || args.uuid.is_some() || args.id.is_some() {
                        client::vm::delete(
                            GetVmArgs {
                                id: args.id,
                                uuid: args.uuid,
                                name: args.name,
                            },
                            cw_node,
                        )
                        .await?;
                    } else if args.state.is_some() || args.account.is_some() {
                        client::vm::delete_many(
                            GetManyVmArgs {
                                vm_state: args.state,
                                account_uuid: args.account,
                            },
                            cw_node,
                        )
                        .await?;
                    }
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
