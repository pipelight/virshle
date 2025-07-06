use crate::cloud_hypervisor::VmState;
use clap::{Args, Parser, Subcommand, ValueEnum, ValueHint};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
    #[command(flatten)]
    pub verbose: Verbosity,

    #[command(flatten)]
    pub current_workgin_node: CurrentWorkingNode,
}

#[derive(Debug, Subcommand, Clone, Eq, PartialEq)]
pub enum Commands {
    /// Init/Ensure system global configuration (openvswitches, directories, database).
    Init,

    /// Operations on templates
    #[command(subcommand)]
    Node(NodeArgs),

    /// Operations on templates
    #[command(subcommand)]
    Template(TemplateArgs),

    /// Operations on virtual machines
    #[command(subcommand)]
    Vm(Crud),
}

#[derive(Debug, Subcommand, Clone, Eq, PartialEq)]
pub enum Crud {
    /// Creates a virtual machine.
    #[command(arg_required_else_help = true)]
    Create(CreateArgs),
    /// Starts/Restart a virtual machine.
    #[command(arg_required_else_help = true)]
    Start(StartArgs),
    /// Removes(destroy) a virtual machine.
    #[command(arg_required_else_help = true)]
    Rm(VmArgs),
    /// Stops a virtual machine.
    #[command(arg_required_else_help = true)]
    Stop(VmArgs),
    /// Parse a virtual machine toml configuration.
    #[command(arg_required_else_help = true)]
    Config(VmArgs),
    /// Inspect a created virtual machine configuration (cloud-hypervisor api).
    #[command(arg_required_else_help = true)]
    Info(VmArgs),

    /// List existing vms.
    #[command()]
    Ls(VmArgs),

    #[command(hide = true)]
    Update(CreateArgs),
}

#[derive(Default, Debug, Args, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CreateArgs {
    #[arg(short, long, value_name="FILE", value_hint=ValueHint::FilePath, 
        conflicts_with = "template",
    )]
    pub file: Option<String>,
    #[arg(short, long, value_name = "TEMPLATE_NAME", conflicts_with = "file")]
    pub template: Option<String>,

    /// Pass user data to VM.
    /// It links the VM to the provided account on node database
    #[arg(short, long, value_name = "USERDATA_FILEPATH")]
    pub user_data: Option<String>,
}

#[derive(Default, Debug, Args, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct VmArgs {
    #[arg(
        long,
        conflicts_with = "id",
        conflicts_with = "uuid",
        value_name = "VM_NAME"
    )]
    pub name: Option<String>,
    #[arg(
        long,
        conflicts_with = "name",
        conflicts_with = "uuid",
        value_name = "VM_ID"
    )]
    pub id: Option<u64>,
    #[arg(
        long,
        conflicts_with = "name",
        conflicts_with = "id",
        value_name = "VM_UUID"
    )]
    pub uuid: Option<Uuid>,

    /// Lookup VM by state.
    #[arg(long, value_name = "VM_STATE")]
    pub state: Option<VmState>,

    /// Lookup VM by account.
    #[arg(long, value_name = "ACCOUNT_UUID")]
    pub account: Option<Uuid>,
}

#[derive(Default, Debug, Args, Clone, Eq, PartialEq, Serialize)]
pub struct StartArgs {
    #[command(flatten)]
    pub vm_args: VmArgs,
    #[arg(
        long,
        num_args(0..=1),
        require_equals = true,
        default_missing_value = "true"
    )]
    pub attach: bool,

    /// Pass user data to VM.
    /// It links the VM to the provided account on node database
    #[arg(short, long, value_name = "USERDATA_FILEPATH")]
    pub user_data: Option<String>,
}

#[derive(Default, Debug, Subcommand, Clone, Eq, PartialEq)]
pub enum TemplateArgs {
    #[default]
    Ls,
}

#[derive(Default, Debug, Subcommand, Clone, Eq, PartialEq)]
pub enum NodeArgs {
    #[default]
    Ls,
    Ping(CurrentWorkingNode),
    Info(CurrentWorkingNode),
    Serve,
}

#[derive(Default, Debug, Args, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CurrentWorkingNode {
    #[arg(long, value_name = "NODE_NAME")]
    pub node: Option<String>,
}
