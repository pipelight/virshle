mod types;
mod utils;
pub use types::*;

use crate::cloud_hypervisor::VmConfigPlus;
use crate::display::VmTable;
use crate::{
    cloud_hypervisor::{Definition, UserData, Vm, VmState, VmTemplate},
    config::{HostCpu, HostDisk, HostRam, Node, NodeInfo, VirshleConfig},
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
use crate::api::{rest::client, rest::method, NodeServer};
use crate::api::{CreateManyVmArgs, CreateVmArgs, GetManyVmArgs, GetVmArgs};
use pipelight_exec::Status;

// Logger
use env_logger::Builder;

// Error Handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{CastError, JsonError, VirshleError, WrapError};

impl Cli {
    pub async fn run() -> Result<()> {
        let cli = Cli::parse();
        Self::switch(cli).await?;
        Ok(())
    }
    pub async fn switch(cli: Cli) -> Result<()> {
        utils::set_tracer(&cli)?;
        utils::set_logger(&cli)?;

        match cli.commands {
            /*
             * Operations on local and remote node
             */
            Commands::Node(args) => match args {
                NodeArgs::Ls(args) => {
                    let node = args.current_workgin_node.node;
                    if node.is_some() {
                        let res = client::node::get_info(node).await?;
                        if args.disk {
                            HostDisk::display(&res).await?;
                        } else if args.ram {
                            HostRam::display(&res).await?;
                        } else if args.cpu {
                            HostCpu::display(&res).await?;
                        } else if args.all {
                            HostDisk::display(&res.clone()).await?;
                            HostRam::display(&res.clone()).await?;
                            HostCpu::display(&res.clone()).await?;
                        } else {
                            Node::display(&res).await?;
                        }
                    } else {
                        let res = client::node::get_info_all().await?;
                        if args.disk {
                            HostDisk::display_many(res).await?;
                        } else if args.ram {
                            HostRam::display_many(res).await?;
                        } else if args.cpu {
                            HostCpu::display_many(res).await?;
                        } else if args.all {
                            HostDisk::display_many(res.clone()).await?;
                            HostRam::display_many(res.clone()).await?;
                            HostCpu::display_many(res.clone()).await?;
                        } else {
                            Node::display_many(res).await?;
                        }
                    }
                }
                NodeArgs::Ping(args) => {
                    let res = client::node::ping(args.node).await?;
                }
                /*
                 * Serve the node rest API and wait for http requests.
                 */
                NodeArgs::Serve => {
                    NodeServer::run().await?;
                }
                /*
                 * Create the required virshle working directories.
                 * Add network ovs entries.
                 * Activate network interfaces.
                 */
                NodeArgs::Init(args) => {
                    if args.all == Some(true) {
                        VirshleConfig::ensure_all().await?;
                    } else {
                        if args.db == Some(true) {
                            VirshleConfig::ensure_database().await?;
                        }
                        if args.net == Some(true) {
                            VirshleConfig::ensure_network().await?;
                        }
                        if args.dir == Some(true) {
                            VirshleConfig::ensure_directories().await?;
                        }
                    }
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
                    let tag = "create";
                    // Set working node
                    let cw_node = args.current_workgin_node.node;
                    let node = Node::unwrap_or_default(cw_node.clone()).await?;

                    let mut user_data = None;
                    if let Some(user_data_filepath) = args.user_data {
                        user_data = Some(UserData::from_file(&user_data_filepath)?);
                    }

                    match args.ntimes {
                        Some(v) => {
                            // Spinner
                            let mut sp = Spinner::new(spinners::Toggle5, "Creating vms...", None);
                            // Create a vm from template.
                            let res: HashMap<Status, Vec<Vm>> = client::vm::create_many(
                                CreateManyVmArgs {
                                    template_name: args.template,
                                    ntimes: args.ntimes,
                                },
                                cw_node.clone(),
                                user_data,
                            )
                            .await?;

                            // Spinner
                            let message = utils::print_response_bulk_op(tag, &node.name, &res)?;
                            sp.stop_and_persist(&message, "");
                        }
                        None => {
                            // Spinner
                            let mut sp = Spinner::new(spinners::Toggle5, "Creating vm...", None);

                            // Create a vm from template.
                            let res: Result<Vm, VirshleError> = client::vm::create(
                                CreateVmArgs {
                                    template_name: args.template,
                                },
                                cw_node.clone(),
                                user_data,
                            )
                            .await;
                            // Spinner
                            let message = utils::print_response_op(tag, &node.name, &res)?;
                            sp.stop_and_persist(&message, "");
                        }
                    }
                }
                Crud::Start(args) => {
                    let tag = "start";
                    // Set working node
                    let cw_node = args.vm_args.current_workgin_node.node.clone();
                    let node = Node::unwrap_or_default(cw_node.clone()).await?;

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
                                let res: Result<Vm, VirshleError> = client::vm::start(
                                    GetVmArgs {
                                        id: args.id,
                                        uuid: args.uuid,
                                        name: args.name,
                                    },
                                    cw_node.clone(),
                                    user_data,
                                )
                                .await;
                                // Spinner
                                let logs = utils::print_response_op(tag, &node.name, &res)?;
                                sp.stop_and_persist(&logs, "");
                            } else if args.state.is_some() || args.account.is_some() {
                                // Spinner
                                let mut sp =
                                    Spinner::new(spinners::Toggle5, "Starting vms...", None);
                                let res: HashMap<Status, Vec<Vm>> = client::vm::start_many(
                                    GetManyVmArgs {
                                        vm_state: args.state,
                                        account_uuid: args.account,
                                    },
                                    cw_node.clone(),
                                    user_data,
                                )
                                .await?;
                                let message =
                                    utils::print_response_bulk_op("start", &node.name, &res)?;
                                sp.stop_and_persist(&message, "");
                            }
                        }
                    };
                }
                Crud::Stop(args) => {
                    let tag = "shutdown";
                    // Set working node
                    let cw_node = args.current_workgin_node.node;
                    let node = Node::unwrap_or_default(cw_node.clone()).await?;

                    if args.name.is_some() || args.uuid.is_some() || args.id.is_some() {
                        // Spinner
                        let mut sp = Spinner::new(spinners::Toggle5, "Shutting down vm...", None);
                        let res: Result<Vm, VirshleError> = client::vm::shutdown(
                            GetVmArgs {
                                id: args.id,
                                uuid: args.uuid,
                                name: args.name,
                            },
                            cw_node.clone(),
                        )
                        .await;
                        // Spinner
                        let message = utils::print_response_op(tag, &node.name, &res)?;
                        sp.stop_and_persist(&message, "");
                    } else if args.state.is_some() || args.account.is_some() {
                        // Spinner
                        let mut sp = Spinner::new(spinners::Toggle5, "Shutting down vms...", None);
                        let res = client::vm::shutdown_many(
                            GetManyVmArgs {
                                vm_state: args.state,
                                account_uuid: args.account,
                            },
                            cw_node.clone(),
                        )
                        .await?;
                        // Spinner
                        let message = utils::print_response_bulk_op(tag, &node.name, &res)?;
                        sp.stop_and_persist(&message, "");
                    }
                }
                Crud::Delete(args) => {
                    let tag = "delete";
                    // Set working node
                    let cw_node = args.current_workgin_node.node;
                    let node = Node::unwrap_or_default(cw_node.clone()).await?;

                    if args.name.is_some() || args.uuid.is_some() || args.id.is_some() {
                        // Spinner
                        let mut sp = Spinner::new(spinners::Toggle5, "Deleting vm...", None);

                        let res: Result<Vm, VirshleError> = client::vm::delete(
                            GetVmArgs {
                                id: args.id,
                                uuid: args.uuid,
                                name: args.name,
                            },
                            cw_node.clone(),
                        )
                        .await;
                        // Spinner
                        let message = utils::print_response_op(tag, &node.name, &res)?;
                        sp.stop_and_persist(&message, "");
                    } else if args.state.is_some() || args.account.is_some() {
                        // Spinner
                        let mut sp = Spinner::new(spinners::Toggle5, "Deleting vms...", None);

                        let res: HashMap<Status, Vec<Vm>> = client::vm::delete_many(
                            GetManyVmArgs {
                                vm_state: args.state,
                                account_uuid: args.account,
                            },
                            cw_node.clone(),
                        )
                        .await?;
                        let message = utils::print_response_bulk_op(tag, &node.name, &res)?;
                        sp.stop_and_persist(&message, "");
                    }
                }
                Crud::Ls(args) => {
                    let cw_node = args.current_workgin_node.node;
                    let table = client::vm::get_info_many(
                        Some(GetManyVmArgs {
                            vm_state: args.state,
                            account_uuid: args.account,
                        }),
                        cw_node,
                    )
                    .await?;
                    match args.format.json {
                        Some(true) => {
                            let json: Vec<(Node, Vec<VmTable>)> = table
                                .iter()
                                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                                .collect();
                            let string = serde_json::to_string_pretty(&json).unwrap();
                            println!("{}", string);
                        }
                        _ => VmTable::display_by_nodes(table).await?,
                    };
                }
                Crud::ChInfo(args) => {
                    // Set working node
                    let cw_node = args.current_workgin_node.node.clone();
                    if args.name.is_some() || args.uuid.is_some() || args.id.is_some() {
                        let res = client::vm::get_ch_info(
                            GetVmArgs {
                                id: args.id,
                                uuid: args.uuid,
                                name: args.name,
                            },
                            cw_node.clone(),
                        )
                        .await?;
                        println!("{:#?}", res);
                    }
                }
                Crud::Info(args) => {
                    // Set working node
                    let cw_node = args.current_workgin_node.node.clone();
                    if args.name.is_some() || args.uuid.is_some() || args.id.is_some() {
                        let res = client::vm::get_info(
                            GetVmArgs {
                                id: args.id,
                                uuid: args.uuid,
                                name: args.name,
                            },
                            cw_node.clone(),
                        )
                        .await?;
                        println!("{:#?}", res);
                    }
                }
                Crud::Definition(args) => {
                    // Set working node
                    let cw_node = args.current_workgin_node.node.clone();
                    if args.name.is_some() || args.uuid.is_some() || args.id.is_some() {
                        let res = client::vm::get_definition(
                            GetVmArgs {
                                id: args.id,
                                uuid: args.uuid,
                                name: args.name,
                            },
                            cw_node.clone(),
                        )
                        .await?;
                        res.print_to_toml()?;
                    }
                }
                Crud::GetVsockPath(args) => {
                    let vm = Vm::get_by_args(&GetVmArgs {
                        id: args.id,
                        uuid: args.uuid,
                        name: args.name,
                    })
                    .await?;
                    let path = vm.get_vsocket()?;
                    println!("{}", path);
                }
                Crud::GetListNames(args) => {
                    let vm = Vm::get_many_by_args(&GetManyVmArgs {
                        vm_state: args.state,
                        ..Default::default()
                    })
                    .await?;
                    let names: Vec<String> = vm.iter().map(|e| e.name.to_owned()).collect();
                    let list = names.join("\n");
                    println!("{}", list);
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
