// Time
use chrono::{NaiveDateTime, TimeDelta};
use human_bytes::human_bytes;

// Colors
use owo_colors::OwoColorize;

// Net
use crossterm::{style::Stylize, terminal::size};
use std::net::IpAddr;
use uuid::Uuid;

// Error Handling
use log::{log_enabled, Level};
use miette::Result;

use tabled::{
    settings::{disable::Remove, object::Columns, Style},
    Table, Tabled,
};

/// Format a Vec<T> to Table.
pub fn default<T>(vec: Vec<T>) -> Result<()>
where
    T: Tabled,
{
    if log_enabled!(Level::Warn) {
        let mut res = Table::new(&vec);
        res.with(Style::rounded());
        println!("{}", res);
    } else {
        let mut res = Table::new(&vec);
        res.with(Remove::column(Columns::last()));
        res.with(Style::rounded());
        println!("{}", res);
    }
    Ok(())
}

pub fn display_duration(delta: &TimeDelta) -> String {
    let mut parsed: String = "".to_owned();
    let delta = delta.abs();
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
    let res = human_bytes(vram.to_owned() as f64);
    format!("{}", res)
}

// Convert cloud-hypervisor ram from MiB.
pub fn display_vram(vram: &u64) -> String {
    let res = human_bytes((vram * u64::pow(1024, 3)) as f64);
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
