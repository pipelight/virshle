#[cfg(test)]
pub mod e2e_tests;

// Virshle daemon http Rest API
mod client;
mod commons;
mod server;

pub use client::Client;
pub use commons::*;
pub use server::Server;
