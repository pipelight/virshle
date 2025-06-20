#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_must_use)]
#![allow(unused_must_use)]

// Virshle daemon http Rest API
pub mod api;
pub use api::NodeServer;
// pub use api::{GrpcClient, RestClient};

// Toml configuration structs.
pub mod config;
pub use config::{Node, NodeInfo};

// Virshle command line
pub mod cli;

// Interact with cloud hypervisor processes and API.
pub mod cloud_hypervisor;
pub use cloud_hypervisor::{Vm, VmState, VmTemplate};

// Methods to do http easily on unix sockets.
// Used to interact with cloud hypervisor
pub mod connection;
pub mod http_request;

// Host network manipulation.
pub mod network;
pub use network::{ip, ovs};

// Stores vm definitions in sqlite database
pub mod database;

// Display virshle types in pretty tables.
pub mod display;
