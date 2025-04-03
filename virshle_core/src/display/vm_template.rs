use super::utils::{display_disks, display_ips, display_vram};
use crate::cloud_hypervisor::{DiskTemplate, VmTemplate};

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fmt;
use tabled::{
    settings::{object::Columns, Style},
    Table, Tabled,
};

// Error Handling
use log::{log_enabled, Level};
use miette::{IntoDiagnostic, Result};
use virshle_error::VirshleError;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub struct VmTemplateTable {
    pub name: String,
    pub vcpu: u64,
    #[tabled(display = "display_vram")]
    pub vram: u64,
    #[tabled(display = "display_disks")]
    pub disk: Option<Vec<DiskTemplate>>,
}

impl VmTemplateTable {
    async fn from(vm: &VmTemplate) -> Result<Self, VirshleError> {
        let table = VmTemplateTable {
            name: vm.name.to_owned(),
            vcpu: vm.vcpu,
            vram: vm.vram,
            disk: vm.disk.to_owned(),
        };
        Ok(table)
    }
}

impl VmTemplateTable {
    pub async fn display(items: Vec<Self>) -> Result<()> {
        let mut res = Table::new(&items);
        res.with(Style::modern_rounded());
        println!("{}", res);
        Ok(())
    }
}
impl VmTemplate {
    pub async fn display(items: Vec<Self>) -> Result<()> {
        let mut table: Vec<VmTemplateTable> = vec![];
        for e in items {
            table.push(VmTemplateTable::from(&e).await?);
        }

        // Default sort templates by ram usage
        table.sort_by(|a, b| a.vram.cmp(&b.vram));

        VmTemplateTable::display(table).await?;
        Ok(())
    }
}
