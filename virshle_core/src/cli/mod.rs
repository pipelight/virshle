mod types;
pub use types::*;

use crate::{
    cloud_hypervisor::{Definition, Vm, VmTemplate},
    config::{Node, VirshleConfig},
    Client, Server,
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
            /*
             * Run the background daemon and wait for http requests.
             */
            Commands::Daemon => {
                Server::run().await?;
            }
            /*
             * Operations on virtual machine templates
             */
            Commands::Node(args) => match args {
                Display::Ls => {
                    let e = VirshleConfig::get()?.get_nodes()?;
                    Node::display(e).await?;
                }
            },
            /*
             * Operations on virtual machine templates
             */
            Commands::Template(args) => match args {
                Display::Ls => {
                    let e = Client::get_all_templates().await?;
                    VmTemplate::display_by_nodes(e).await?;
                }
            },
            /*
             * Operations on virtual machines
             */
            Commands::Vm(args) => match args {
                Crud::Create(args) => {
                    // Create a vm from strict definition in file.
                    if let Some(file) = args.file {
                        let mut vm = Vm::from_file(&file)?;
                        vm.create().await?;
                    }
                    // Create a vm from template.
                    if let Some(name) = args.template {
                        let template = VirshleConfig::get()?.get_template(&name)?;
                        let mut vm = Vm::from(&template);
                        vm.create().await?;
                    }
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
                    if let Some(name) = args.resource.name {
                        let mut vm = Vm::get_by_name(&name).await?;
                        if args.attach {
                            vm.attach()?.start().await?;
                        } else {
                            vm.start().await?;
                        }
                    } else if let Some(id) = args.resource.id {
                        let mut vm = Vm::get_by_id(&id).await?;
                        if args.attach {
                            vm.attach()?.start().await?;
                        } else {
                            vm.start().await?;
                        }
                    }
                }
                Crud::Stop(args) => {
                    if let Some(name) = args.name {
                        let vm = Vm::get_by_name(&name).await?;
                        vm.shutdown().await?;
                    } else if let Some(id) = args.id {
                        let vm = Vm::get_by_id(&id).await?;
                        vm.shutdown().await?;
                    }
                }
                Crud::Ls(args) => {
                    let e = Client::get_all_vm_w_args(args).await?;
                    Vm::display_by_nodes(e).await?;
                }
                Crud::Rm(args) => {
                    if let Some(name) = args.name {
                        let vm = Vm::get_by_name(&name).await?;
                        vm.delete().await?;
                    } else if let Some(id) = args.id {
                        let vm = Vm::get_by_id(&id).await?;
                        vm.delete().await?;
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
