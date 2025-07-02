pub mod fake;
pub mod kea;
pub mod lease;

use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

// Reexports
pub use fake::FakeDhcp;
pub use kea::KeaDhcp;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DhcpType {
    Fake(FakeDhcp),
    Kea(KeaDhcp),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IpPool {
    subnet: IpNet,
    range: [IpAddr; 2],
}
