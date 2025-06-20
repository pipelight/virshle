#![allow(refining_impl_trait_reachable)]

pub mod ip;
pub mod utils;
pub use std::str::FromStr;

pub mod interface;
pub mod ovs;

// Query dhcp server for ipv6/ipv4 leases.
pub mod dhcp;

pub use interface::{Bridge, InterfaceManager, InterfaceState};
