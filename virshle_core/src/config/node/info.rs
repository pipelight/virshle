use crate::Vm;
use sysinfo::{Disks, System};

use crate::config::MANAGED_DIR;
use crate::config::{MAX_CPU_RESERVATION, MAX_DISK_RESERVATION, MAX_RAM_RESERVATION};

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
    // Return node saturation index.
    pub async fn get_saturation_index(&self) -> Result<f64, VirshleError> {
        let info = &self.host_info;

        let weight_disk = 10.0;
        let weight_ram = 7.0;
        let weight_cpu = 4.0;

        let index = (info.disk.saturation_index().await? * weight_disk
            + info.ram.saturation_index().await? * weight_ram
            + info.cpu.saturation_index().await? * weight_cpu)
            / (weight_disk + weight_ram + weight_cpu);

        Ok(index)
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq)]
pub struct HostInfo {
    pub name: String,
    // Stored as Bytes.
    pub ram: HostRam,
    pub cpu: HostCpu,
    pub disk: HostDisk,
}
impl HostInfo {
    pub async fn get() -> Result<Self, VirshleError> {
        let name = System::host_name().unwrap_or("unknown".to_owned());
        Ok(HostInfo {
            name,
            ram: HostRam::get().await?,
            cpu: HostCpu::get().await?,
            disk: HostDisk::get().await?,
        })
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq)]
pub struct HostRam {
    pub total: u64,
    pub free: u64,
    pub used: u64,
    pub available: u64,
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
    pub async fn get() -> Result<Self, VirshleError> {
        let mut s = System::new_all();
        s.refresh_memory();
        // Ram
        let ram = HostRam {
            total: s.total_memory(),
            free: s.free_memory(),
            available: s.available_memory(),
            used: s.used_memory(),
            reserved: Self::get_reserved().await?,
        };
        Ok(ram)
    }
    pub async fn get_percentage_reserved(&self) -> Result<f64, VirshleError> {
        let res = self.reserved as f64 / self.total as f64 * 100.0;
        Ok(res)
    }
    pub async fn saturation_index(&self) -> Result<f64, VirshleError> {
        let res = self.get_percentage_reserved().await? / MAX_RAM_RESERVATION;
        Ok(res)
    }
    // RAM saturation
    pub async fn is_saturated(&self) -> Result<bool, VirshleError> {
        Ok(self.reserved as f64 / self.total as f64 * 100.0 >= MAX_RAM_RESERVATION)
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq)]
pub struct HostCpu {
    pub number: u64,
    pub usage: f64,
    // The number of cpu reserved for VMs.
    pub reserved: u64,
}
impl HostCpu {
    /// Get the amount of cpu that is reserved for VMs
    /// wheter they are running, and using it or not.
    /// (from vm definitions in the node database)
    pub async fn get() -> Result<Self, VirshleError> {
        let mut s = System::new_all();
        s.refresh_cpu_all();

        let cpu = HostCpu {
            number: s.cpus().len() as u64,
            usage: Self::get_usage().await?,
            reserved: HostCpu::get_reserved().await?,
        };
        Ok(cpu)
    }
    pub async fn get_usage() -> Result<f64, VirshleError> {
        let mut s = System::new_all();
        s.refresh_cpu_all();
        // Need to update twice because of usage computation.
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        s.refresh_cpu_all();

        // Cpu
        let total_usage: f64 = s.cpus().iter().map(|e| e.cpu_usage() as f64).sum();
        let n_cpus = s.cpus().len() as f64;
        let average_usage = total_usage / n_cpus;
        Ok(average_usage)
    }
    pub async fn get_reserved() -> Result<u64, VirshleError> {
        let vms = Vm::get_all().await?;
        let n_cpus: u64 = vms.iter().map(|e| e.vcpu).sum();
        Ok(n_cpus)
    }
    pub async fn get_percentage_reserved(&self) -> Result<f64, VirshleError> {
        let res = self.reserved as f64 / self.number as f64 * 100.0;
        Ok(res)
    }
    pub async fn saturation_index(&self) -> Result<f64, VirshleError> {
        let res = self.get_percentage_reserved().await? / MAX_CPU_RESERVATION;
        Ok(res)
    }
    // CPU saturation
    pub async fn is_saturated(&self) -> Result<bool, VirshleError> {
        Ok(self.reserved as f64 / self.number as f64 * 100.0 >= MAX_CPU_RESERVATION)
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialOrd, PartialEq)]
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

    pub async fn get_percentage_reserved(&self) -> Result<f64, VirshleError> {
        let res = self.reserved as f64 / self.size as f64 * 100.0;
        Ok(res)
    }
    pub async fn saturation_index(&self) -> Result<f64, VirshleError> {
        let res = self.get_percentage_reserved().await? / MAX_DISK_RESERVATION;
        Ok(res)
    }
    // Disk saturation
    pub async fn is_saturated(&self) -> Result<bool, VirshleError> {
        Ok(self.reserved as f64 / self.size as f64 * 100.0 >= MAX_DISK_RESERVATION)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_info() -> Result<()> {
        let host = HostInfo::get().await?;
        println!("{:#?}", host);
        Ok(())
    }

    #[tokio::test]
    async fn test_df() -> Result<()> {
        HostDisk::get().await?;
        Ok(())
    }
    #[tokio::test]
    async fn test_cpu() -> Result<()> {
        let cpu = HostCpu::get().await?;
        println!("{:#?}", cpu);
        Ok(())
    }
}
