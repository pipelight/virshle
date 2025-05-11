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
    pub ram: u64,
    pub cpu: u64,
}

impl Host {
    pub fn get_info() -> Result<Self, VirshleError> {
        let mut s = System::new_all();
        s.refresh_memory();
        s.refresh_cpu_all();

        let ram = s.total_memory();
        let cpu = s.cpus().len() as u64;
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
