#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_must_use)]
#![allow(unused_must_use)]

// Virshle daemon http Rest API
pub mod rest_api;
pub use rest_api::NodeServer;
// pub use api::{GrpcClient, RestClient};

// Toml configuration structs.
pub mod config;
pub use config::{Node, NodeInfo};

// Virshle command line
pub mod cli;

// Interact with host network configuration.
pub mod network;

// Interact with cloud hypervisor processes and API.
pub mod hypervisor;
pub use hypervisor::disk::utils::{human_bytes, reverse_human_bytes};
pub use hypervisor::Account;
pub use hypervisor::{Vm, VmInfo, VmState, VmTemplate};

// Methods to do http easily on unix sockets.
// Used to interact with cloud hypervisor
pub mod connection;
pub mod http_request;

pub mod exec;

// Stores vm definitions in sqlite database
pub mod database;

// Display virshle types in pretty tables.
pub mod display;
pub use display::VmTable;
