pub mod ip;
pub use std::str::FromStr;

pub mod ovs;

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
