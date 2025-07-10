use crate::Vm;
use sysinfo::{Disks, System};

use crate::config::MANAGED_DIR;
use std::path::Path;

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
        let host_info = HostInfo::get().await?;
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
    pub disk: HostDisk,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct HostRam {
    pub total: u64,
    pub free: u64,
    // The ram allocated to VMs
    pub reserved: u64,
}
impl HostRam {
    /*
     * Get the amount of cpu that is reserved for VMs
     * wheter they are running, and using it or not.
     * (from vm definitions in the node database)
     */
    pub async fn get_reserved() -> Result<u64, VirshleError> {
        let vms = Vm::get_all().await?;
        let total_ram: u64 = vms.iter().map(|e| e.vram * u64::pow(1024, 3)).sum();
        Ok(total_ram)
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct HostCpu {
    pub number: u64,
    pub usage: u64,
    // The number of cpu reserved for VMs.
    pub reserved: u64,
}
impl HostCpu {
    /*
     * Get the amount of cpu that is reserved for VMs
     * wheter they are running, and using it or not.
     * (from vm definitions in the node database)
     */
    pub async fn get_reserved() -> Result<u64, VirshleError> {
        let vms = Vm::get_all().await?;
        let n_cpus: u64 = vms.iter().map(|e| e.vcpu).sum();
        Ok(n_cpus)
    }
}

impl HostInfo {
    pub async fn get() -> Result<Self, VirshleError> {
        let mut s = System::new_all();
        s.refresh_memory();
        s.refresh_cpu_all();

        let name = System::host_name().unwrap_or("unknown".to_owned());
        // Ram
        let ram = HostRam {
            total: s.total_memory(),
            free: s.free_memory(),
            reserved: HostRam::get_reserved().await?,
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
            reserved: HostCpu::get_reserved().await?,
        };

        // Disk
        let disk = HostDisk::get().await?;

        Ok(HostInfo {
            name,
            ram,
            cpu,
            disk,
        })
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct HostDisk {
    pub size: u64,
    pub used: u64,
    pub available: u64,
    pub reserved: u64,
    // The disk space reserved for Vm storage.
}

impl HostDisk {
    pub async fn get() -> Result<Self, VirshleError> {
        let managed_path = Path::new(&MANAGED_DIR).canonicalize().unwrap();

        let disks = Disks::new_with_refreshed_list();
        let disks = disks.list();

        // let disks = read_fs_list().unwrap();
        for disk in disks {
            let ancestors: Vec<String> = managed_path
                .ancestors()
                .map(|e| e.to_str().unwrap().to_owned())
                .collect();
            let mount_point = &disk.mount_point().to_str().unwrap().to_owned();
            if ancestors.contains(mount_point) {
                let disk_info = HostDisk {
                    size: disk.total_space(),
                    available: disk.available_space(),
                    used: disk.total_space() - disk.available_space(),
                    reserved: Self::get_reserved().await?,
                };
                return Ok(disk_info);
            }
        }
        let message = format!("Couldn't find the disk used by vmm.");
        let help = format!("Are you sure this path exists?");
        let err = LibError::builder().msg(&message).help(&help).build();
        Err(err.into())
    }

    pub async fn get_reserved() -> Result<u64, VirshleError> {
        let vms = Vm::get_all().await?;
        let n_cpus: u64 = vms
            .iter()
            .map(|e| e.disk.iter().map(|d| d.get_size().unwrap()).sum::<u64>())
            .sum();
        Ok(n_cpus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_info() -> Result<()> {
        let host = HostInfo::get().await?;
        println!("{:?}", host);
        Ok(())
    }

    #[tokio::test]
    async fn test_df() -> Result<()> {
        HostDisk::get().await?;
        Ok(())
    }
}
