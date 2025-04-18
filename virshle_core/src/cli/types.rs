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

    /// Init/Ensure system global configuration (openvswitches, directories, database).
    Init,

    // TODO: Declarative docker compose style
    #[command(hide = true)]
    Up(File),
    #[command(hide = true)]
    Down(File),

    /// Operations on networks
    #[command(subcommand, hide = true)]
    Net(Crud),

    /// Operations on templates
    #[command(subcommand)]
    Template(Display),

    /// Operations on virtual machines
    #[command(subcommand)]
    Vm(Crud),
}

#[derive(Debug, Args, Clone, Eq, PartialEq)]
pub struct File {
    #[arg(short, long, value_name="FILE", value_hint=ValueHint::FilePath, 
        conflicts_with = "template",
    )]
    pub file: Option<String>,
    #[arg(short, long, value_name = "TEMPLATE_NAME", conflicts_with = "file")]
    pub template: Option<String>,
}
#[derive(Debug, Subcommand, Clone, Eq, PartialEq)]
pub enum Crud {
    /// Creates a virtual machine.
    #[command(arg_required_else_help = true)]
    Create(File),
    /// Removes(destroy) a virtual machine.
    #[command(arg_required_else_help = true)]
    Rm(Resource),
    /// Starts/Restart a virtual machine.
    #[command(arg_required_else_help = true)]
    Start(Resource),
    /// Stops a virtual machine.
    #[command(arg_required_else_help = true)]
    Stop(Resource),
    /// Parse a virtual machine toml configuration.
    #[command(arg_required_else_help = true)]
    Config(Resource),
    /// Inspect a created virtual machine configuration (cloud-hypervisor api).
    #[command(arg_required_else_help = true)]
    Info(Resource),

    /// List existing vms.
    Ls,

    #[command(hide = true)]
    Update(File),
}
#[derive(Debug, Args, Clone, Eq, PartialEq)]
pub struct Resource {
    #[arg(long, conflicts_with = "id")]
    pub name: Option<String>,
    #[arg(long, conflicts_with = "name")]
    pub id: Option<u64>,
}
#[derive(Debug, Subcommand, Clone, Eq, PartialEq)]
pub enum Display {
    Ls,
}
#[derive(Debug, Args, Clone, Eq, PartialEq)]
pub struct ResourceUuid {
    uuid: String,
}
