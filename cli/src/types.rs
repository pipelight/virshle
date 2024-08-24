use virshle_core::{
    display,
    resources::{Net, Vm},
};

// Logger
use env_logger::Builder;
use log::LevelFilter;
// Error Handling
use miette::Result;

use clap::{Args, Parser, Subcommand, ValueEnum, ValueHint};
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,

    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,
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
            Commands::Create(args) => {}
            Commands::Delete(args) => {}
            Commands::List(args) => match args.r#type {
                ResourceType::VM => {
                    display(Vm::get_all()?)?;
                }
                ResourceType::Net => {
                    display(Net::get_all()?)?;
                }
            },
            _ => {}
        };
        Ok(())
    }
}

#[derive(Debug, Subcommand, Clone, Eq, PartialEq)]
pub enum Commands {
    #[command(arg_required_else_help = true)]
    Create(File),
    #[command(arg_required_else_help = true)]
    Delete(Resource),
    List(Resource),
}

#[derive(Debug, Args, Clone, Eq, PartialEq)]
pub struct Resource {
    #[clap(value_enum)]
    r#type: ResourceType,
    name: Option<String>,
}

#[derive(Debug, Default, ValueEnum, Clone, Eq, PartialEq)]
pub enum ResourceType {
    #[default]
    // List domains
    VM,
    // List networks
    Net,
}

#[derive(Debug, Clone, Eq, PartialEq, Parser)]
pub struct File {
    pub file: Option<String>,
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
        let e = "virshle list vm";
        let os_str: Vec<&str> = e.split(' ').collect();
        let cli = Cli::parse_from(os_str);
        Cli::switch(cli)?;
        Ok(())
    }
    #[test]
    fn get_networks() -> Result<()> {
        println!("");
        let e = "virshle list net";
        let os_str: Vec<&str> = e.split(' ').collect();
        let cli = Cli::parse_from(os_str);
        Cli::switch(cli)?;
        Ok(())
    }
}
