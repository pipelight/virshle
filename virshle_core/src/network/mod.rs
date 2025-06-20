pub mod ip;
pub mod utils;
pub use std::str::FromStr;

pub mod interface;
pub mod ovs;

// Query dhcp server for ipv6/ipv4 leases.
pub mod dhcp;

// Error handling
use miette::{IntoDiagnostic, Result};

use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InterfaceState {
    Up,
    Down,
}

impl InterfaceState {
    fn from_str(s: &str) -> Result<Option<InterfaceState>> {
        let cased = s.to_case(Case::Title);
        let res = match cased.as_str() {
            "Up" => Some(InterfaceState::Up),
            "Down" => Some(InterfaceState::Down),
            "Unknown" | _ => None,
        };
        Ok(res)
    }
}
