use crate::{
    // resources,
    // resources::{create, Net, ResourceType, Secret, Vm},
    cloud_hypervisor::{Definition, Net, Vm},
    config::VirshleConfig,
    convert,
    Api,
};
use std::fs;
use std::path::Path;

// Logger
use env_logger::Builder;
use log::LevelFilter;

// Error Handling
use miette::{IntoDiagnostic, Result};

use clap::{Args, Parser, Subcommand, ValueEnum, ValueHint};
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
    #[command(flatten)]
    pub verbose: Verbosity,
}
#[derive(Debug, Subcommand, Clone, Eq, PartialEq)]
pub enum Commands {
    Prune,
    Daemon,

    // Declarative docker compose style
    Up(File),
    Down(File),

    // Crud classic style
    #[command(subcommand)]
    Vm(Crud),
    #[command(subcommand)]
    Net(Crud),
    #[command(subcommand)]
    Secret(CrudUuid),
}
#[derive(Debug, Args, Clone, Eq, PartialEq)]
pub struct File {
    file: String,
}
#[derive(Debug, Subcommand, Clone, Eq, PartialEq)]
pub enum Crud {
    Create(File),
    Update(File),
    Start(Resource),
    Stop(Resource),
    Inspect(Resource),
    Rm(Resource),
    Ls,
}
#[derive(Debug, Args, Clone, Eq, PartialEq)]
pub struct Resource {
    name: String,
}
#[derive(Debug, Subcommand, Clone, Eq, PartialEq)]
pub enum CrudUuid {
    Rm(ResourceUuid),
    Create(File),
    Ls,
}
#[derive(Debug, Args, Clone, Eq, PartialEq)]
pub struct ResourceUuid {
    uuid: String,
}

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
            Commands::Prune => {
                // remove unused managed files
                // resources::clean()?;
            }
            Commands::Daemon => {
                Api::run().await?;
            }
            Commands::Up(args) => {
                let mut def = Definition::from_file(&args.file)?;
                def.create_all().await?;
                def.start_all().await?;
            }
            Commands::Down(args) => {
                let def = Definition::from_file(&args.file)?;
                def.delete_all().await?;
            }
            Commands::Vm(args) => match args {
                Crud::Create(args) => {
                    let template_map = config.get_vm_template()?;
                    let template = template_map.get(&args.file);
                    if let Some(template) = template {
                        let vm = Vm::from(template);
                        vm.create().await?;
                    } else {
                        let vm = Vm::from_file(&args.file)?;
                        vm.create().await?;
                    }
                }
                Crud::Inspect(resource) => {
                    let vm = Vm::get_by_name(&resource.name).await?;
                    vm.get_info().await?;
                }
                Crud::Start(resource) => {
                    let mut vm = Vm::get_by_name(&resource.name).await?;
                    vm.start().await?;
                }
                Crud::Stop(resource) => {
                    Vm::get_by_name(&resource.name).await?.shutdown().await?;
                }
                Crud::Ls => {
                    Vm::display(Vm::get_all().await?).await?;
                }
                Crud::Rm(resource) => {
                    let vm = Vm::get_by_name(&resource.name).await?;
                    vm.delete().await?;
                }
                _ => {}
            },
            Commands::Net(args) => match args {
                Crud::Ls => {
                    Net::display(Net::get_all().await?).await?;
                }
                Crud::Rm(resource) => {
                    let net = Net::get_by_name(&resource.name).await?;
                    net.delete().await?;
                }
                Crud::Create(args) => {
                    let net = Net::from_file(&args.file)?;
                    net.create().await?;
                }
                Crud::Start(resource) => {
                    let net = Net::get_by_name(&resource.name).await?;
                    net.start().await?;
                }
                _ => {}
            },
            Commands::Secret(args) => match args {
                CrudUuid::Ls => {
                    // display::default(Secret::get_all()?)?;
                }
                CrudUuid::Rm(resource) => {
                    // Secret::delete(&resource.uuid)?;
                }
                CrudUuid::Create(args) => {
                    // Secret::set(&args.file)?;
                }
            },
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
    async fn get_domains() -> Result<()> {
        println!("");
        let e = "virshle vm ls";
        let os_str: Vec<&str> = e.split(' ').collect();
        let cli = Cli::parse_from(os_str);
        Cli::switch(cli).await?;
        Ok(())
    }
    #[tokio::test]
    async fn get_networks() -> Result<()> {
        println!("");
        let e = "virshle net ls";
        let os_str: Vec<&str> = e.split(' ').collect();
        let cli = Cli::parse_from(os_str);
        Cli::switch(cli).await?;
        Ok(())
    }
}
