use super::{info::HostInfo, Node};

// Error Handling
use crate::{
    config::{MAX_CPU_RESERVATION, MAX_DISK_RESERVATION, MAX_RAM_RESERVATION},
    VmTemplate,
};
use log::{info, warn};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

impl Node {
    pub fn is_best(name: &str, url: &str) -> Result<Self, VirshleError> {
        // Order by weight

        // Order by space left for Vm creation.
        let e = Node {
            name: name.to_owned(),
            url: url.to_owned(),
            weight: 0,
        };

        // Select random in purged list

        Ok(e)
    }
    pub async fn is_cpu_saturated(&self, vm_template: &VmTemplate) -> Result<bool, VirshleError> {
        let info = HostInfo::get().await?;
        if info.cpu.reserved as f64 / info.cpu.number as f64 * 100.0 >= MAX_CPU_RESERVATION {
            return Ok(true);
        }
        Ok(false)
    }
    // Ram saturation
    pub async fn is_ram_saturated(&self, vm_template: &VmTemplate) -> Result<bool, VirshleError> {
        let info = HostInfo::get().await?;
        if info.ram.available as f64 / info.ram.total as f64 * 100.0 >= MAX_RAM_RESERVATION {
            return Ok(true);
        }
        Ok(false)
    }
    // Disk saturation
    pub async fn is_disk_saturated(&self, vm_template: &VmTemplate) -> Result<bool, VirshleError> {
        let info = HostInfo::get().await?;
        if let Some(disks) = &vm_template.disk {
            let total_size: u64 = disks
                .iter()
                .map(|e| e.get_size().unwrap())
                .reduce(|acc, x| acc + x)
                .unwrap();
            if info.disk.available as f64 / total_size as f64 * 100.0 >= MAX_DISK_RESERVATION {
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub async fn is_disk_reserved(&self, vm_template: &VmTemplate) -> Result<bool, VirshleError> {
        let info = HostInfo::get().await?;
        if let Some(disks) = &vm_template.disk {
            let total_vm_size: u64 = disks
                .iter()
                .map(|e| e.get_size().unwrap())
                .reduce(|acc, x| acc + x)
                .unwrap();
            if total_vm_size as f64 / info.disk.size as f64 * 100.0 >= MAX_DISK_RESERVATION {
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub async fn is_saturated(&self, vm_template: &VmTemplate) -> Result<bool, VirshleError> {
        let info = HostInfo::get().await?;

        Ok(false)
    }
}
