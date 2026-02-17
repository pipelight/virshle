use crate::config::DiskTemplate;
use crate::hypervisor::Disk;
use serde::{Deserialize, Serialize};

use crate::utils::display::{display_some_bool, display_some_bytes};
use tabled::Tabled;

// Display
use human_bytes::human_bytes;

// Error Handling
use miette::Result;
use virshle_error::VirshleError;

#[derive(Default, Debug, Serialize, Deserialize, Clone, Ord, PartialOrd, Eq, PartialEq, Tabled)]
pub struct DiskInfo {
    pub name: String,
    pub path: String,
    #[tabled(display = "display_some_bytes")]
    pub size: Option<u64>,
    #[tabled(display = "display_some_bool")]
    pub readonly: Option<bool>,
}

impl DiskInfo {
    pub fn from_vec(e: &Vec<Disk>) -> Result<Vec<Self>, VirshleError> {
        let mut res = vec![];
        for disk in e {
            let info = DiskInfo::from(&disk)?;
            res.push(info);
        }
        Ok(res)
    }
    pub fn from(e: &Disk) -> Result<Self, VirshleError> {
        let info = DiskInfo {
            name: e.name.clone(),
            path: e.path.clone(),
            size: e.get_size().ok(),
            readonly: e.readonly,
        };
        Ok(info)
    }
    pub fn from_template(e: &DiskTemplate) -> Result<Self, VirshleError> {
        let info = DiskInfo {
            name: e.name.clone(),
            path: e.path.clone(),
            size: e.get_size().ok(),
            readonly: e.readonly,
        };
        Ok(info)
    }
}

impl DiskInfo {
    pub fn display_some_vec(disks: &Option<Vec<DiskInfo>>) -> String {
        let mut res = "".to_owned();
        if let Some(disks) = disks {
            let mut summary: Vec<String> = vec![];
            for e in disks {
                if let Some(size) = e.size {
                    let size = human_bytes(size as f64);

                    let oneline = format!("{} -> {} ({size})", e.name, e.path);
                    summary.push(oneline);
                } else {
                    let oneline = format!("{} -> {}", e.name, e.path);
                    summary.push(oneline);
                }
            }
            res = summary.join("\n");
        }
        res
    }
    pub fn display_some_vec_short(disks: &Option<Vec<DiskInfo>>) -> String {
        let mut res = "".to_owned();
        if let Some(disks) = disks {
            let mut summary: Vec<String> = vec![];
            for e in disks {
                if let Some(size) = e.size {
                    let size = human_bytes(size as f64);

                    let oneline = format!("{} ({size})", e.name);
                    summary.push(oneline);
                } else {
                    let oneline = format!("{}", e.name,);
                    summary.push(oneline);
                }
            }
            res = summary.join("\n");
        }
        res
    }
}
