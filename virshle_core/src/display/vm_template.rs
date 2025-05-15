use super::utils::{display_disks, display_ips, display_vram};
use crate::cloud_hypervisor::{DiskTemplate, VmTemplate};
use crate::config::Node;
use crate::connection::Uri;

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub fn display_w_header(items: Vec<Self>, header: &str) -> Result<(), VirshleError> {
        println!("\n{}", header);
        let mut res = Table::new(&items);
        res.with(Style::modern_rounded());
        println!("{}", res);
        Ok(())
    }
    pub async fn display(items: Vec<Self>) -> Result<()> {
        let mut res = Table::new(&items);
        res.with(Style::modern_rounded());
        println!("{}", res);
        Ok(())
    }
}
impl VmTemplate {
    pub async fn display_by_nodes(items: HashMap<Node, Vec<Self>>) -> Result<(), VirshleError> {
        // Convert vm to pretty printable type
        let mut tables: HashMap<Node, Vec<VmTemplateTable>> = HashMap::new();
        for (node, vms) in items {
            let mut vms_table: Vec<VmTemplateTable> = vec![];
            for vm in vms {
                let e = VmTemplateTable::from(&vm).await?;
                vms_table.push(e);
            }
            tables.insert(node, vms_table);
        }

        // Display vm by nodes with table header
        for (node, table) in tables {
            let name = node.name.bright_purple().bold().to_string();
            let header: String = match Uri::new(&node.url)? {
                Uri::SshUri(e) => format!(
                    "{name} on {}@{}",
                    e.user.yellow().bold(),
                    e.host.green().bold()
                ),
                Uri::LocalUri(e) => format!("{name} on {}", "localhost".green().bold()),
            };
            VmTemplateTable::display_w_header(table, &header);
        }

        Ok(())
    }
    pub async fn display(items: Vec<Self>) -> Result<()> {
        let mut table: Vec<VmTemplateTable> = vec![];
        for e in items {
            table.push(VmTemplateTable::from(&e).await?);
        }

        // Default sort templates by vram size
        table.sort_by(|a, b| a.vram.cmp(&b.vram));

        VmTemplateTable::display(table).await?;
        Ok(())
    }
}
