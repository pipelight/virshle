use std::fs;
use std::path::Path;
use virshle_core::{
    convert, display, resources,
    resources::{create, Net, ResourceType, Secret, Vm},
};

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
    pub fn run() -> Result<()> {
        let cli = Cli::parse();
        Self::switch(cli)?;
        Ok(())
    }
    pub fn switch(cli: Cli) -> Result<()> {
        // Set verbosity
        let verbosity = cli.verbose.log_level_filter();
        std::env::set_var("VIRSHLE_LOG", verbosity.to_string().to_lowercase());
        Builder::from_env("VIRSHLE_LOG").init();

        match cli.commands {
            Commands::Prune => {
                // remove unused managed files
                resources::clean()?;
            }
            Commands::Create(args) => {
                let toml = fs::read_to_string(args.file).into_diagnostic()?;
                create(&toml)?;
            }
            Commands::Vm(args) => match args {
                Crud::Create(args) => {
                    Vm::set(&args.file)?;
                }
                Crud::Start(resource) => {
                    Vm::get(&resource.name)?.start()?;
                }
                Crud::Shutdown(resource) => {
                    Vm::get(&resource.name)?.shutdown()?;
                }
                Crud::Ls => {
                    display::vm(Vm::get_all()?)?;
                }
                Crud::Rm(resource) => {
                    Vm::get(&resource.name)?.delete()?;
                }
            },
            Commands::Net(args) => match args {
                Crud::Ls => {
                    display::default(Net::get_all()?)?;
                }
                Crud::Rm(resource) => {
                    Net::get_by_name(&resource.name)?.delete()?;
                }
                Crud::Create(args) => {
                    Net::from_path(&args.file)?;
                }
                _ => {}
            },
            Commands::Secret(args) => match args {
                CrudUuid::Ls => {
                    display::default(Secret::get_all()?)?;
                }
                CrudUuid::Rm(resource) => {
                    Secret::delete(&resource.uuid)?;
                }
                CrudUuid::Create(args) => {
                    Secret::set(&args.file)?;
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

    #[test]
    fn parse_command_line() -> Result<()> {
        println!("");
        let e = "virshle --help";
        let os_str: Vec<&str> = e.split(' ').collect();
        let cli = Cli::parse_from(os_str);
        Cli::switch(cli)?;
        Ok(())
    }

    #[test]
    fn get_domains() -> Result<()> {
        println!("");
        let e = "virshle vm ls";
        let os_str: Vec<&str> = e.split(' ').collect();
        let cli = Cli::parse_from(os_str);
        Cli::switch(cli)?;
        Ok(())
    }
    #[test]
    fn get_networks() -> Result<()> {
        println!("");
        let e = "virshle net ls";
        let os_str: Vec<&str> = e.split(' ').collect();
        let cli = Cli::parse_from(os_str);
        Cli::switch(cli)?;
        Ok(())
    }
}
