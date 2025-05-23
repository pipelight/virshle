use crate::Vm;
use sysinfo::System;

use crate::connection::ConnectionState;
use serde::{Deserialize, Serialize};

// Error handling
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct NodeInfo {
    pub host_info: HostInfo,
    pub virshle_info: VirshleInfo,
}
impl NodeInfo {
    pub async fn get() -> Result<Self, VirshleError> {
        let host_info = HostInfo::get()?;
        let virshle_info = VirshleInfo::get().await?;

        Ok(NodeInfo {
            host_info,
            virshle_info,
        })
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct VirshleInfo {
    // Number of vm on node.
    pub num_vm: u64,
}
impl VirshleInfo {
    pub async fn get() -> Result<Self, VirshleError> {
        let num_vm = Vm::get_all().await?.len() as u64;
        Ok(VirshleInfo { num_vm })
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct HostInfo {
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

impl HostInfo {
    pub fn get() -> Result<Self, VirshleError> {
        let mut s = System::new_all();
        s.refresh_memory();
        s.refresh_cpu_all();

        let name = System::host_name().unwrap_or("unknown".to_owned());
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

        Ok(HostInfo { name, ram, cpu })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_info() -> Result<()> {
        let host = HostInfo::get()?;
        println!("{:?}", host);
        Ok(())
    }
}
