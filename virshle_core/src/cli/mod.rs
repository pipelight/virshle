mod types;
pub use types::*;

use crate::{
    cloud_hypervisor::{Definition, Vm, VmTemplate},
    config::VirshleConfig,
    Server,
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
        std::env::set_var("VIRSHLE_LOG", verbosity.to_string().to_lowercase());
        Builder::from_env("VIRSHLE_LOG").init();

        // Get config
        let config = VirshleConfig::get()?;

        match cli.commands {
            /*
             * Static infra generation from file
             * (like vagrant or terraform)
             */
            Commands::Up(args) => {}
            Commands::Down(args) => {}
            /*
             * Remove unused managed files
             * resources::clean()?;
             */
            Commands::Prune => {}

            /*
             * Run the background daemon and wait for http requests.
             */
            Commands::Daemon => {
                Server::run().await?;
            }
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
                        let template = config.get_template(&name)?;
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
                    if let Some(name) = args.name {
                        let mut vm = Vm::get_by_name(&name).await?;
                        if args.attach {
                            vm.attach()?.start().await?;
                        } else {
                            vm.start().await?;
                        }
                    } else if let Some(id) = args.id {
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
                Crud::Ls => {
                    Vm::display(Vm::get_all().await?).await?;
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
            Commands::Template(args) => match args {
                Display::Ls => {
                    let templates: Vec<VmTemplate> =
                        config.get_vm_templates()?.into_values().collect();
                    VmTemplate::display(templates).await?;
                }
            },
            Commands::Init => {
                VirshleConfig::init().await?;
            }
            _ => {}
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
