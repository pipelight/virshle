#[cfg(test)]
mod tests;

mod types;
mod utils;
pub use types::*;

use virshle_core::{
    config::{Config, Definition, Node, VmTemplate},
    hypervisor::{UserData, Vm, VmState, VmTable},
    peer::{HostCpu, HostDisk, HostRam, NodeInfo, Peer},
    utils::testing,
};

use clap::Parser;
use indexmap::IndexMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

// Spinners
use owo_colors::OwoColorize;
use spinoff::{spinners, Color, Spinner};

// Rest API client
use virshle_rest::{Client, Server};

use pipelight_exec::Status;

// Error Handling
use miette::Result;
use virshle_error::VirshleError;

impl Cli {
    pub async fn run() -> Result<(), VirshleError> {
        let cli = Cli::parse();
        Self::switch(cli).await?;
        Ok(())
    }
    pub async fn switch(cli: Cli) -> Result<(), VirshleError> {
        let verbosity = tracing::Level::from_str(&cli.verbose.to_string()).unwrap();

        testing::tracer().verbosity(verbosity).set()?;
        testing::logger().verbosity(verbosity).set()?;

        let config = Config::get()?;
        let mut client = Client::new().config(&config).build()?.api().await?;

        match cli.commands {
            /*
             * Operations on local node.
             */
            Commands::Peer(args) => match args {
                PeerArgs::Ls(args) => {
                    let node = args.current_workgin_node.peer;
                    let node_info = client.node()?.get_info().exec().await?;

                    let peer = Peer::default();
                    let mut res = (peer, node_info);

                    if node.is_some() {
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
                            Peer::display(&res).await?;
                        }
                    } else {
                        let res = client.peer().get_info().exec().await?;
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
                            Peer::display_many(res).await?;
                        }
                    }
                }
                PeerArgs::Ping(args) => {
                    let res = client.peer().ping().exec().await?;
                } /*
                   * Serve the node rest API and wait for http requests.
                   */
            },
            Commands::Node(args) => match args {
                NodeArgs::Serve => {
                    Server::new().config(&config).build()?.serve().await?;
                }
                /*
                 * Create the required virshle working directories.
                 * Add network ovs entries.
                 * Activate network interfaces.
                 */
                NodeArgs::Init(args) => {
                    if args.all == Some(true) {
                        config.init().ensure_all().await?;
                    } else {
                        if args.db == Some(true) {
                            config.init().database().await?;
                        }
                        if args.dir == Some(true) {
                            config.init().directories().await?;
                        }
                        if args.net == Some(true) {
                            config.init().network().await?;
                        }
                    }
                }
            },
            // Operations on virtual machine templates
            //
            Commands::Template(args) => match args {
                TemplateArgs::Ls => {
                    let res = client.template().get().exec().await?;
                    VmTemplate::display_by_peers(res).await?;
                }
            },
            // Operations on virtual machines
            //
            Commands::Vm(args) => match args {
                Crud::Create(args) => {
                    let tag = "create";
                    // Set working node
                    let cw_node = args.current_workgin_node.peer;
                    let node: Peer = config.peer().maybe_alias(cw_node).get()?;

                    let mut user_data = None;
                    if let Some(user_data_path) = args.user_data {
                        user_data = Some(UserData::from_file(&user_data_path)?);
                    }

                    match args.ntimes {
                        Some(v) => {
                            // Spinner
                            let mut sp = Spinner::new(spinners::Toggle5, "Creating vms...", None);
                            // Create a vm from template.
                            let res: Vec<VmTable> = client
                                .vm()
                                .create()
                                .many()
                                .maybe_template(args.template)
                                .maybe_n(args.ntimes)
                                .maybe_user_data(user_data)
                                .exec()
                                .await?;

                            // cw_node.clone(),
                            // Spinner
                            // let message = utils::print_response_bulk_op(tag, &node.name, &res)?;
                            // sp.stop_and_persist(&message, "");
                        }
                        None => {
                            // Spinner
                            let mut sp = Spinner::new(spinners::Toggle5, "Creating vm...", None);

                            // Create a vm from template.
                            let res: Result<VmTable, VirshleError> = client
                                .vm()
                                .create()
                                .one()
                                .maybe_template(args.template)
                                .maybe_user_data(user_data)
                                .exec()
                                .await;

                            // Spinner
                            // let message = utils::print_response_op(tag, &node.name, &res)?;
                            // sp.stop_and_persist(&message, "");
                        }
                    }
                }
                Crud::Start(args) => {
                    let tag = "start";

                    // Set working node
                    let cw_node = args.vm_args.current_workgin_node.peer.clone();
                    let node: Peer = config.peer().maybe_alias(cw_node).get()?;

                    let user_data: Option<UserData> = match args.user_data {
                        Some(path) => Some(UserData::from_file(&path)?),
                        None => None,
                    };
                    let methods = Server::new().config(&config).build()?.api()?;

                    if args.vm_args.name.is_some()
                        || args.vm_args.uuid.is_some()
                        || args.vm_args.id.is_some()
                    {
                        // Spinner
                        let mut sp = Spinner::new(spinners::Toggle5, "Starting vm...", None);

                        // Rest API
                        let res: VmTable = client
                            .vm()
                            .start()
                            .one()
                            .maybe_id(args.vm_args.id)
                            .maybe_uuid(args.vm_args.uuid)
                            .maybe_name(args.vm_args.name.clone())
                            .maybe_user_data(user_data.clone())
                            .exec()
                            .await?;

                        match args.attach {
                            true => {
                                // Bypass rest API,
                                // and run on local node direcly.
                                methods
                                    .vm()
                                    .start()
                                    .one()
                                    .maybe_id(args.vm_args.id)
                                    .maybe_uuid(args.vm_args.uuid)
                                    .maybe_name(args.vm_args.name.clone())
                                    .maybe_user_data(user_data.clone())
                                    .exec()
                                    .await?;
                            }
                            _ => {}
                        }

                        // Spinner
                        // let logs = utils::print_response_op(tag, &node.name, &res)?;
                        // sp.stop_and_persist(&logs, "");
                    } else if args.vm_args.state.is_some() || args.vm_args.account.is_some() {
                        // Spinner
                        let mut sp = Spinner::new(spinners::Toggle5, "Starting vms...", None);
                        let res: IndexMap<Peer, IndexMap<Status, Vec<VmTable>>> = client
                            .vm()
                            .start()
                            .many()
                            .maybe_state(args.vm_args.state)
                            .maybe_account(args.vm_args.account)
                            .exec()
                            .await?;
                        // let message =
                        //     utils::print_response_bulk_op("start", &node.name, &res)?;
                        // sp.stop_and_persist(&message, "");
                    }
                }
                Crud::Stop(args) => {
                    let tag = "shutdown";

                    // Set working node
                    let cw_node = args.current_workgin_node.peer;
                    let node: Peer = config.peer().maybe_alias(cw_node).get()?;

                    if args.name.is_some() || args.uuid.is_some() || args.id.is_some() {
                        // Spinner
                        let mut sp = Spinner::new(spinners::Toggle5, "Shutting down vm...", None);
                        let _res: VmTable = client
                            .vm()
                            .shutdown()
                            .one()
                            .maybe_id(args.id)
                            .maybe_uuid(args.uuid)
                            .maybe_name(args.name)
                            .exec()
                            .await?;
                        // Spinner
                        // let message = utils::print_response_op(tag, &node.name, &res)?;
                        // sp.stop_and_persist(&message, "");
                    } else if args.state.is_some() || args.account.is_some() {
                        // Spinner
                        let mut sp = Spinner::new(spinners::Toggle5, "Shutting down vms...", None);
                        let res = client
                            .vm()
                            .shutdown()
                            .many()
                            .maybe_state(args.state)
                            .maybe_account(args.account)
                            .exec()
                            .await?;
                        // Spinner
                        // let message = utils::print_response_bulk_op(tag, &node.name, &res)?;
                        // sp.stop_and_persist(&message, "");
                    }
                }
                Crud::Delete(args) => {
                    let tag = "delete";

                    // Set working node
                    let cw_node = args.current_workgin_node.peer;
                    let node: Peer = config.peer().maybe_alias(cw_node).get()?;

                    if args.name.is_some() || args.uuid.is_some() || args.id.is_some() {
                        // Spinner
                        let mut sp = Spinner::new(spinners::Toggle5, "Deleting vm...", None);

                        let _res: VmTable = client
                            .vm()
                            .delete()
                            .one()
                            .maybe_id(args.id)
                            .maybe_uuid(args.uuid)
                            .maybe_name(args.name)
                            .exec()
                            .await?;

                        // Spinner
                        // let message = utils::print_response_op(tag, &node.name, &res)?;
                        // sp.stop_and_persist(&message, "");
                    } else if args.state.is_some() || args.account.is_some() {
                        // Spinner
                        let mut sp = Spinner::new(spinners::Toggle5, "Deleting vms...", None);

                        let res = client
                            .vm()
                            .delete()
                            .many()
                            .maybe_state(args.state)
                            .maybe_account(args.account)
                            .exec()
                            .await?;
                        // let message = utils::print_response_bulk_op(tag, &node.name, &res)?;
                        // sp.stop_and_persist(&message, "");
                    }
                }
                Crud::Ls(args) => {
                    let cw_node = args.current_workgin_node.peer;
                    if args.name.is_some() || args.uuid.is_some() || args.id.is_some() {
                        let table: VmTable = client
                            .vm()
                            .get()
                            .one()
                            .maybe_id(args.id)
                            .maybe_uuid(args.uuid)
                            .maybe_name(args.name)
                            .maybe_alias(cw_node)
                            .exec()
                            .await?;
                        if args.format.ron == Some(true) {
                            println!("{:#?}", table);
                        } else if args.format.json == Some(true) {
                            // TODO: do not work on HashMap.
                            // let string = serde_json::to_string_pretty(&table).unwrap();
                            // println!("{}", string);
                        } else {
                            VmTable::display(&vec![table])?
                        }
                    } else {
                        let table = client
                            .vm()
                            .get()
                            .many()
                            .maybe_state(args.state)
                            .maybe_account(args.account)
                            .exec()
                            .await?;
                        if args.format.ron == Some(true) {
                            println!("{:#?}", table);
                        } else if args.format.json == Some(true) {
                            // TODO: do not work on HashMap.
                            // let string = serde_json::to_string_pretty(&table).unwrap();
                            // println!("{}", string);
                        } else {
                            VmTable::display_by_peer(&table).await?
                        }
                    }
                }
                Crud::Info(args) => {
                    // Set working node
                    let cw_node = args.current_workgin_node.peer.clone();
                    if args.name.is_some() || args.uuid.is_some() || args.id.is_some() {
                        let res = client
                            .vm()
                            .vmm()
                            .info()
                            .maybe_id(args.id)
                            .maybe_uuid(args.uuid)
                            .maybe_name(args.name)
                            .exec()
                            .await?;
                        println!("{:#?}", res);
                        // res.print_to_toml()?;
                    }
                }
                Crud::GetVsockPath(args) => {
                    let vm = Vm::database()
                        .await?
                        .one()
                        .maybe_id(args.id)
                        .maybe_name(args.name)
                        .maybe_uuid(args.uuid)
                        .get()
                        .await?;
                    let path = vm.get_vsocket()?;
                    println!("{}", path);
                }
                Crud::GetListNames(args) => {
                    let vms = Vm::database()
                        .await?
                        .many()
                        .maybe_vm_state(args.state)
                        .maybe_account_uuid(args.account)
                        .get()
                        .await?;
                    let names: Vec<String> = vms.iter().map(|e| e.name.to_owned()).collect();
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
