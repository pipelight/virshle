#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_must_use)]
#![allow(unused_must_use)]

// Virshle daemon http Rest API
pub mod http_api;
pub use http_api::Api;

// Toml configuration structs.
pub mod config;

// Virshle command line
pub mod cli;

// Interact with cloud hypervisor processes and API.
pub mod cloud_hypervisor;
// Methods to do http easily on unix sockets.
// Used to interact with cloud hypervisor
pub mod http_cli;

// Host network manipulation.
pub mod network;
pub use network::Ip;

// Stores vm definitions in sqlite database
pub mod database;

// Display virshle types in pretty tables.
pub mod display;
