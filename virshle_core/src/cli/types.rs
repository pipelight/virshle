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
}

#[derive(Default, Debug, Args, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CurrentWorkingNode {
    #[arg(long, value_name = "NODE_NAME")]
    pub node: Option<String>,
}

#[derive(Debug, Subcommand, Clone, Eq, PartialEq)]
pub enum Commands {
    /// Operations on nodes
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
    #[command(alias = "on", arg_required_else_help = true)]
    Start(StartArgs),
    /// Removes(destroy) a virtual machine.
    #[command(alias = "rm", arg_required_else_help = true)]
    Delete(VmArgs),
    /// Stops a virtual machine.
    #[command(alias = "off", arg_required_else_help = true)]
    Stop(VmArgs),

    /// Parse a virtual machine toml configuration.
    #[command(arg_required_else_help = true)]
    Config(VmArgs),

    /// Inspect a created virtual machine configuration (cloud-hypervisor api).
    #[command(arg_required_else_help = true)]
    Info(VmArgs),
    /// Inspect a created virtual machine configuration (cloud-hypervisor api).
    #[command(arg_required_else_help = true)]
    ChInfo(VmArgs),
    /// Return the path fo vm vsock
    #[command(arg_required_else_help = true, hide = true)]
    GetVsockPath(VmArgs),
    /// Return the path fo vm vsock
    #[command(arg_required_else_help = true, hide = true)]
    Definition(VmArgs),

    /// List existing vms.
    #[command()]
    Ls(VmArgs),

    #[command(hide = true)]
    Update(CreateArgs),
}
#[derive(Default, Debug, Args, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct InitArgs {
    /// Initialize everything
    #[arg(long,num_args(0..=1),
        require_equals = true,
        default_missing_value = "true"
    )]
    pub all: Option<bool>,
    /// Best effort to configure host network.
    #[arg(long,num_args(0..=1),
        require_equals = true,
        default_missing_value = "true"
    )]
    pub net: Option<bool>,
    /// Best effort to migrate old database or create fresh.
    #[arg(long,num_args(0..=1),
        require_equals = true,
        default_missing_value = "true"
    )]
    pub db: Option<bool>,
    /// Create directories on host.
    #[arg(long,num_args(0..=1),
        require_equals = true,
        default_missing_value = "true"
    )]
    pub dir: Option<bool>,
}

#[derive(Default, Debug, Args, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CreateArgs {
    #[arg(short, long, value_name="FILE", value_hint=ValueHint::FilePath, 
        conflicts_with = "template",
    )]
    pub file: Option<String>,
    #[arg(short, long, value_name = "TEMPLATE_NAME", conflicts_with = "file")]
    pub template: Option<String>,

    // It links the VM to the provided account on the local node database.
    /// Pass user data to VM.
    #[arg(short, long, value_name = "USERDATA_FILEPATH")]
    pub user_data: Option<String>,

    #[command(flatten)]
    pub current_workgin_node: CurrentWorkingNode,
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

    /// Lookup VM by account_uuid.
    #[arg(long, value_name = "ACCOUNT_UUID")]
    pub account: Option<Uuid>,

    #[command(flatten)]
    pub current_workgin_node: CurrentWorkingNode,
}

#[derive(Default, Debug, Args, Clone, Eq, PartialEq, Serialize)]
pub struct StartArgs {
    #[arg(
        long,
        num_args(0..=1),
        require_equals = true,
        default_missing_value = "true"
    )]
    pub attach: bool,

    // It links the VM to the provided account on the local node database.
    /// Pass user data to VM.
    #[arg(short, long, value_name = "USERDATA_FILEPATH")]
    pub user_data: Option<String>,

    #[command(flatten)]
    pub vm_args: VmArgs,
}

#[derive(Default, Debug, Subcommand, Clone, Eq, PartialEq)]
pub enum TemplateArgs {
    #[default]
    Ls,
}

#[derive(Debug, Subcommand, Clone, Eq, PartialEq)]
pub enum NodeArgs {
    /// Init/Ensure system global configuration (openvswitches, directories, database).
    #[command(arg_required_else_help = true)]
    Init(InitArgs),

    Ls(NodeLsArgs),
    Ping(CurrentWorkingNode),
    Serve,
}

#[derive(Default, Debug, Args, Clone, Eq, PartialEq, Serialize)]
pub struct NodeLsArgs {
    #[arg(
        long,
        num_args(0..=1),
        require_equals = true,
        default_missing_value = "true"
    )]
    pub disk: bool,
    #[arg(
        long,
        num_args(0..=1),
        require_equals = true,
        default_missing_value = "true"
    )]
    pub cpu: bool,
    #[arg(
        long,
        num_args(0..=1),
        require_equals = true,
        default_missing_value = "true"
    )]
    pub ram: bool,
    #[arg(
        long,
        num_args(0..=1),
        require_equals = true,
        default_missing_value = "true"
    )]
    pub all: bool,

    #[command(flatten)]
    pub current_workgin_node: CurrentWorkingNode,
}
