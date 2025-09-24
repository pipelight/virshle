use super::node::{CpuTable, HostDiskTable, RamTable};

// Time
use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};

use owo_colors::OwoColorize;
use std::net::IpAddr;
use uuid::Uuid;

use crate::config::{MAX_CPU_RESERVATION, MAX_DISK_RESERVATION, MAX_RAM_RESERVATION};

use crossterm::{execute, style::Stylize, terminal::size};

use crate::cloud_hypervisor::disk::utils::human_bytes;
use crate::cloud_hypervisor::DiskInfo;

pub fn display_duration(delta: &TimeDelta) -> String {
    let mut parsed: String = "".to_owned();
    let days = delta.num_days();
    let hours = delta.num_hours();
    let minutes = delta.num_minutes();
    if days > 0 {
        parsed += &format!("{days} days");
    } else if hours > 0 {
        parsed += &format!("{hours} hours");
    } else {
        parsed += &format!("{minutes} minutes");
    }
    parsed
}
// Convert from B.
pub fn display_some_bytes(bytes: &Option<u64>) -> String {
    if let Some(bytes) = bytes {
        display_bytes(&bytes)
    } else {
        format!("")
    }
}
pub fn display_bytes(vram: &u64) -> String {
    let res = human_bytes(vram).unwrap();
    format!("{}", res)
}

// Convert host ram from B.
pub fn display_some_ram(ram: &Option<u64>) -> String {
    if let Some(ram) = ram {
        display_ram(&ram)
    } else {
        format!("")
    }
}
pub fn display_ram(ram: &u64) -> String {
    let res = human_bytes(ram).unwrap();
    format!("{}", res)
}
// Convert cloud-hypervisor ram from MiB.
pub fn display_vram(vram: &u64) -> String {
    let res = human_bytes(&(vram * u64::pow(1024, 3))).unwrap();
    format!("{}", res)
}

pub fn display_some_num(num: &Option<u64>) -> String {
    if let Some(num) = num {
        format!("{}", num)
    } else {
        format!("")
    }
}
pub fn display_some_bool(b: &Option<bool>) -> String {
    if let Some(b) = b {
        format!("{}", b)
    } else {
        format!("")
    }
}

pub fn display_some_datetime(date: &Option<NaiveDateTime>) -> String {
    if let Some(date) = date {
        display_datetime(&date)
    } else {
        format!("")
    }
}
pub fn display_datetime(date: &NaiveDateTime) -> String {
    let res = date.format("%m-%d-%Y %H:%M");
    format!("{}", res)
}

pub fn display_some_disks(disks: &Option<Vec<DiskInfo>>) -> String {
    let mut res = "".to_owned();
    if let Some(disks) = disks {
        let mut summary: Vec<String> = vec![];
        for e in disks {
            if let Some(size) = e.size {
                let size = human_bytes(&size).unwrap();

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
pub fn display_some_disks_short(disks: &Option<Vec<DiskInfo>>) -> String {
    let mut res = "".to_owned();
    if let Some(disks) = disks {
        let mut summary: Vec<String> = vec![];
        for e in disks {
            if let Some(size) = e.size {
                let size = human_bytes(&size).unwrap();

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
pub fn display_ips(ips: &Option<Vec<IpAddr>>) -> String {
    let mut res = "".to_owned();
    if let Some(ips) = ips {
        let ips: Vec<String> = ips.iter().map(|e| e.to_string()).collect();
        res = ips.join("\n");
    }
    format!("{}", res)
}

pub fn display_id(id: &Option<u64>) -> String {
    if let Some(id) = id {
        format!("{}", id)
    } else {
        return "".to_owned();
    }
}
pub fn display_account_uuid(uuid: &Option<Uuid>) -> String {
    if let Some(uuid) = uuid {
        format!("{}", uuid)
    } else {
        format!("")
    }
}

pub fn make_progress_bar(percentage: &f64, max: Option<f64>) -> String {
    let (cols, _) = size().unwrap();
    let max = max.unwrap_or(100.0);

    let progress = match cols >= 100 {
        true => {
            let bar_total_size = cols as f64 / 10.0;
            let n_chars = (bar_total_size * percentage / max).ceil();
            let n_empty_chars = ((bar_total_size) * (max - percentage) / max).ceil();
            let adv = "#".repeat(n_chars as usize);
            let nadv = " ".repeat(n_empty_chars as usize);

            let progress = format!("[{adv}{nadv}] {percentage:.1}%");
            progress
        }
        false => {
            let progress = format!("{percentage:.1}%");
            progress
        }
    };
    progress
}

pub fn add_color_used(string: &str, percentage: &f64) -> String {
    if percentage < &100_f64 {
        format!("{}", string.green())
    } else if percentage < &200_f64 {
        format!("{}", string.yellow())
    } else {
        format!("{}", string.red())
    }
}

pub fn display_some_percentage_used(percentage: &Option<f64>) -> String {
    if let Some(percentage) = percentage {
        display_percentage_used(&percentage)
    } else {
        "".to_owned()
    }
}
pub fn display_percentage_used(percentage: &f64) -> String {
    let progress = make_progress_bar(percentage, None);
    if percentage < &80_f64 {
        format!("{}", progress.green())
    } else if percentage < &90_f64 {
        format!("{}", progress.yellow())
    } else {
        format!("{}", progress.red())
    }
}

impl CpuTable {
    pub fn display_some_cpu_percentage_reserved(percentage: &Option<f64>) -> String {
        if let Some(percentage) = percentage {
            Self::display_cpu_percentage_reserved(&percentage)
        } else {
            "".to_owned()
        }
    }
    pub fn display_cpu_percentage_reserved(percentage: &f64) -> String {
        let max = MAX_CPU_RESERVATION;
        let progress = make_progress_bar(percentage, Some(max));
        Self::add_color_reserved(&progress, percentage)
    }
    pub fn add_color_reserved(string: &str, percentage: &f64) -> String {
        if percentage < &200_f64 {
            format!("{}", string.green())
        } else if percentage < &250_f64 {
            format!("{}", string.yellow())
        } else {
            format!("{}", string.red())
        }
    }
    pub fn display_some_cpu_reserved(num: &Option<u64>, percentage: &Option<f64>) -> String {
        if num.is_some() && percentage.is_some() {
            format!(
                "{}",
                Self::add_color_reserved(&num.unwrap().to_string(), &percentage.unwrap())
            )
        } else {
            format!("")
        }
    }
}

impl RamTable {
    pub fn display_some_ram_percentage_reserved(percentage: &Option<f64>) -> String {
        if let Some(percentage) = percentage {
            Self::display_ram_percentage_reserved(&percentage)
        } else {
            "".to_owned()
        }
    }
    pub fn display_ram_percentage_reserved(percentage: &f64) -> String {
        let max = MAX_RAM_RESERVATION;
        let progress = make_progress_bar(percentage, Some(max));
        Self::add_color_reserved(&progress, percentage)
    }
    pub fn display_some_ram_reserved(num: &Option<u64>, percentage: &Option<f64>) -> String {
        if num.is_some() && percentage.is_some() {
            let num = human_bytes(&num.unwrap()).unwrap();
            format!("{}", Self::add_color_reserved(&num, &percentage.unwrap()))
        } else {
            format!("")
        }
    }
    pub fn display_some_ram_percentage_used(percentage: &Option<f64>) -> String {
        if let Some(percentage) = percentage {
            Self::display_ram_percentage_used(&percentage)
        } else {
            "".to_owned()
        }
    }
    pub fn display_ram_percentage_used(percentage: &f64) -> String {
        let max = MAX_RAM_RESERVATION;
        let progress = make_progress_bar(percentage, None);
        Self::add_color_used(&progress, percentage)
    }
    pub fn display_some_ram_used(num: &Option<u64>, percentage: &Option<f64>) -> String {
        if num.is_some() && percentage.is_some() {
            let num = human_bytes(&num.unwrap()).unwrap();
            format!("{}", Self::add_color_used(&num, &percentage.unwrap()))
        } else {
            format!("")
        }
    }
    pub fn add_color_reserved(string: &str, percentage: &f64) -> String {
        if percentage < &100_f64 {
            format!("{}", string.green())
        } else if percentage < &200_f64 {
            format!("{}", string.yellow())
        } else {
            format!("{}", string.red())
        }
    }
    pub fn add_color_used(string: &str, percentage: &f64) -> String {
        if percentage < &50_f64 {
            format!("{}", string.green())
        } else if percentage < &80_f64 {
            format!("{}", string.yellow())
        } else {
            format!("{}", string.red())
        }
    }
}
impl HostDiskTable {
    pub fn display_some_disk_percentage_reserved(percentage: &Option<f64>) -> String {
        if let Some(percentage) = percentage {
            Self::display_disk_percentage_reserved(&percentage)
        } else {
            "".to_owned()
        }
    }
    pub fn display_disk_percentage_reserved(percentage: &f64) -> String {
        let max = MAX_DISK_RESERVATION;
        let progress = make_progress_bar(percentage, Some(max));
        Self::add_color_reserved(&progress, percentage)
    }

    pub fn display_some_disk_reserved(num: &Option<u64>, percentage: &Option<f64>) -> String {
        if num.is_some() && percentage.is_some() {
            let num = human_bytes(&num.unwrap()).unwrap();
            format!("{}", Self::add_color_reserved(&num, &percentage.unwrap()))
        } else {
            format!("")
        }
    }
    pub fn display_some_disk_percentage_used(percentage: &Option<f64>) -> String {
        if let Some(percentage) = percentage {
            Self::display_disk_percentage_used(&percentage)
        } else {
            "".to_owned()
        }
    }
    pub fn display_disk_percentage_used(percentage: &f64) -> String {
        let max = MAX_DISK_RESERVATION;
        let progress = make_progress_bar(percentage, Some(max));
        Self::add_color_used(&progress, percentage)
    }

    pub fn display_some_disk_used(num: &Option<u64>, percentage: &Option<f64>) -> String {
        if num.is_some() && percentage.is_some() {
            let num = human_bytes(&num.unwrap()).unwrap();
            format!("{}", Self::add_color_used(&num, &percentage.unwrap()))
        } else {
            format!("")
        }
    }
    pub fn add_color_reserved(string: &str, percentage: &f64) -> String {
        if percentage < &80_f64 {
            format!("{}", string.green())
        } else if percentage < &95_f64 {
            format!("{}", string.yellow())
        } else {
            format!("{}", string.red())
        }
    }
    pub fn add_color_used(string: &str, percentage: &f64) -> String {
        if percentage < &80_f64 {
            format!("{}", string.green())
        } else if percentage < &95_f64 {
            format!("{}", string.yellow())
        } else {
            format!("{}", string.red())
        }
    }
}
