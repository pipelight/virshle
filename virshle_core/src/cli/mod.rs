mod types;
pub use types::*;

use crate::api::{rest::client, rest::method, NodeServer};
use crate::display::VmTable;
use crate::{
    cloud_hypervisor::{Definition, UserData, Vm, VmTemplate},
    config::{Node, VirshleConfig},
};

use clap::Parser;
use std::fs;
use std::path::Path;

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
                    client::vm::create(&args).await?;
                }
                Crud::Info(args) => {
                    if let Some(name) = args.name {
                        let vm = Vm::get_by_name(&name).await?;
                        vm.to_toml()?;
                        vm.get_info().await?;
                    } else if let Some(id) = args.id {
                        let vm = Vm::get_by_id(&id).await?;
                        vm.to_toml()?;
                        vm.get_info().await?;
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
                    client::vm::shutdown(&args).await?;
                }
                Crud::Ls(args) => {
                    let e = client::vm::get_all(&args).await?;
                    VmTable::display_by_nodes(e).await?;
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
