use crate::display::display_vram;
use human_bytes::human_bytes;
use sysinfo::System;

use serde::{Deserialize, Serialize};

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Host {
    pub name: String,
    // Stored as Bytes.
    pub ram: HostRam,
    pub cpu: HostCpu,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct HostRam {
    pub total: u64,
    pub free: u64,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct HostCpu {
    pub number: u64,
    pub usage: u64,
}

impl Host {
    pub fn get_info() -> Result<Self, VirshleError> {
        let mut s = System::new_all();
        s.refresh_memory();
        s.refresh_cpu_all();

        // Ram
        let ram = HostRam {
            total: s.total_memory(),
            free: s.free_memory(),
        };

        // Cpu
        let average_usage = s
            .cpus()
            .iter()
            .map(|e| e.cpu_usage())
            .reduce(|acc, x| acc + x)
            .unwrap()
            / (s.cpus().len() as f32);

        let cpu = HostCpu {
            number: s.cpus().len() as u64,
            usage: average_usage as u64,
        };

        let name = System::host_name().unwrap_or("unknown".to_owned());

        let host = Host { name, ram, cpu };

        Ok(host)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_info() -> Result<()> {
        let host = Host::get_info()?;
        println!("{:?}", host);
        Ok(())
    }
}
