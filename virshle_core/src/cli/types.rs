use crate::{
    // resources,
    // resources::{create, Net, ResourceType, Secret, Vm},
    cloud_hypervisor::Vm,
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
    Create(File),
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
    Start(Resource),
    Shutdown(Resource), // Replace with stop
    // Stop(Resource),
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

        match cli.commands {
            Commands::Prune => {
                // remove unused managed files
                // resources::clean()?;
            }
            Commands::Daemon => {
                Api::run().await?;
            }
            Commands::Create(args) => {
                Vm::from_file(&args.file)?;
            }
            Commands::Vm(args) => match args {
                Crud::Create(args) => {
                    let mut vm = Vm::from_file(&args.file)?;
                    vm.create().await?;
                }
                Crud::Start(resource) => {
                    let mut vm = Vm::get_by_name(&resource.name).await?;
                    vm.start().await?;
                }
                Crud::Shutdown(resource) => {
                    // Vm::get(&resource.name).await?.shutdown().await?;
                }
                Crud::Ls => {
                    Vm::display(Vm::get_all().await?).await?;
                }
                Crud::Rm(resource) => {
                    let mut vm = Vm::get_by_name(&resource.name).await?;
                    vm.delete().await?;
                }
            },
            Commands::Net(args) => match args {
                Crud::Ls => {
                    // display::default(Net::get_all()?)?;
                }
                Crud::Rm(resource) => {
                    // Net::get_by_name(&resource.name)?.delete()?;
                }
                Crud::Create(args) => {
                    // Net::from_path(&args.file)?;
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
