use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::network::dhcp::{FakeDhcp, IpPool, KeaDhcp};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DhcpType {
    Fake(FakeDhcpConfig),
    Kea(KeaDhcpConfig),
}

// Fake dhcp.
// Virshle populate vm with random static ips.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FakeDhcpConfig {
    pub pool: HashMap<String, IpPool>,
}
impl Into<FakeDhcp> for FakeDhcpConfig {
    fn into(self) -> FakeDhcp {
        (&self).into()
    }
}
impl Into<FakeDhcp> for &FakeDhcpConfig {
    fn into(self) -> FakeDhcp {
        FakeDhcp {
            pool: self.pool.clone(),
        }
    }
}

// Kea dhcp.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeaDhcpConfig {
    pub url: Option<String>,
    pub suffix: Option<String>,
}
impl Default for KeaDhcpConfig {
    fn default() -> Self {
        Self {
            url: Some("tcp://localhost:5547".to_owned()),
            suffix: Some("vm".to_owned()),
        }
    }
}
