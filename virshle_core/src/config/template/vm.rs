use crate::config::DiskTemplate;
use crate::hypervisor::{Disk, DiskInfo, Vm, VmConfigPlus};
use crate::peer::Peer;

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tabled::{settings::Style, Table, Tabled};
use uuid::Uuid;
use virshle_network::Uri;

// Globals
use crate::config::MANAGED_DIR;

// Error Handling
use miette::Result;
use virshle_error::{CastError, TomlError, VirshleError};

/// A partial Vm definition, with optional disk, network...
/// All those usually mandatory fields will be handled by virshle with
/// autoconfigured default.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct VmTemplate {
    pub name: String,
    pub vcpu: u64,
    pub vram: String,
    pub uuid: Option<Uuid>,
    pub disk: Option<Vec<DiskTemplate>>,
    pub net: Option<Vec<VmNet>>,
    pub config: Option<VmConfigPlus>,
}

impl VmTemplate {
    pub fn from_file(file_path: &str) -> Result<Self, VirshleError> {
        let string = fs::read_to_string(file_path)?;
        Self::from_toml(&string)
    }
    pub fn from_toml(string: &str) -> Result<Self, VirshleError> {
        let res = toml::from_str::<Self>(string);
        let item: Self = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlError(TomlError::new(e, string));
                return Err(err.into());
            }
        };
        Ok(item)
    }
}
impl VmTemplate {
    pub async fn display_by_peers(items: HashMap<Peer, Vec<Self>>) -> Result<(), VirshleError> {
        // Convert vm to pretty printable type
        let mut tables: HashMap<Peer, Vec<VmTemplateTable>> = HashMap::new();
        for (peer, vms) in items {
            let mut vms_table: Vec<VmTemplateTable> = vec![];
            for vm in vms {
                let e = VmTemplateTable::from(&vm)?;
                vms_table.push(e);
            }
            tables.insert(peer, vms_table);
        }

        // Display vm by nodes with table header
        for (peer, table) in tables {
            let name = peer.alias()?.bright_purple().bold().to_string();
            let header: String = match Uri::new(&peer.url)? {
                Uri::SshUri(e) => format!(
                    "{name} on {}@{}",
                    e.user.yellow().bold(),
                    e.host.green().bold()
                ),
                Uri::LocalUri(e) => format!("{name} on {}", "localhost".green().bold()),
                Uri::TcpUri(e) => format!(
                    "{name} on {}{}",
                    e.host.green().bold(),
                    e.port.blue().bold()
                ),
            };
            VmTemplateTable::display_w_header(table, &header);
        }

        Ok(())
    }
    pub async fn display(items: Vec<Self>) -> Result<()> {
        let mut table: Vec<VmTemplateTable> = vec![];
        for e in items {
            table.push(VmTemplateTable::from(&e)?);
        }

        // Default sort templates by vram size
        table.sort_by(|a, b| a.vram.cmp(&b.vram));

        VmTemplateTable::display(table).await?;
        Ok(())
    }
}
impl TryInto<Vm> for VmTemplate {
    type Error = VirshleError;
    fn try_into(self) -> Result<Vm, Self::Error> {
        (&self).try_into()
    }
}
impl TryInto<Vm> for &VmTemplate {
    type Error = VirshleError;
    fn try_into(self) -> Result<Vm, Self::Error> {
        let mut vm = Vm {
            vcpu: self.vcpu.clone(),
            vram: self.vram.clone(),
            net: self.net.clone(),
            ..Default::default()
        };
        ensure_directories(&self, &mut vm)?;
        create_disks(&self, &mut vm)?;
        Ok(vm)
    }
}
/*
* Ensure vm storage directories exists on host.
*/
pub fn ensure_directories(template: &VmTemplate, vm: &mut Vm) -> Result<(), VirshleError> {
    let directories = [
        format!("{MANAGED_DIR}/vm/{}", vm.uuid),
        format!("{MANAGED_DIR}/vm/{}/disk", vm.uuid),
        format!("{MANAGED_DIR}/vm/{}/net", vm.uuid),
    ];
    for directory in directories {
        let path = Path::new(&directory);
        if !path.exists() {
            fs::create_dir_all(&directory)?;
        }
    }
    Ok(())
}

/*
* Copy template disks (if some)
* to vm storage directory and set file permissions.
*/
pub fn create_disks(template: &VmTemplate, vm: &mut Vm) -> Result<(), VirshleError> {
    if let Some(disks) = &template.disk {
        for disk in disks {
            let source = DiskTemplate::shellexpand(&disk.path)?;
            let target = format!("{MANAGED_DIR}/vm/{}/disk/{}", vm.uuid, disk.name);

            // Create disk on host drive
            let file = fs::File::create(&target)?;
            fs::copy(&source, &target)?;

            // Set permissions
            let mut perms = fs::metadata(&target)?.permissions();
            perms.set_mode(0o766);
            fs::set_permissions(&target, perms)?;

            // Push disk path to vm def
            vm.disk.push(Disk {
                name: disk.name.clone(),
                path: target,
                readonly: Some(false),
            })
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct VmNet {
    pub name: String,
    #[serde(rename = "type")]
    pub _type: NetType,
}
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NetType {
    Vhost(Vhost),
    Tap(Tap),
    // Can't pass macvtap through the Cloud-hypervisor http API.
    // Must be deprecated because actual ch implementation sucks!
    #[serde(rename = "macvtap")]
    MacVTap(Tap),
}
impl fmt::Display for NetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = match self {
            NetType::Vhost(v) => "vhost".to_owned(),
            NetType::Tap(v) => "tap".to_owned(),
            NetType::MacVTap(v) => "macvtap".to_owned(),
        };
        write!(f, "{}", string)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Tap {
    // Set static mac address or random if none.
    pub mac: Option<String>,
    // Request a static ipv4 ip on the interface.
    pub ip: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Vhost {
    // Set static mac address or random if none.
    pub mac: Option<String>,
    // Request a static ipv4 ip on the interface.
    pub ip: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Tabled)]
pub struct VmTemplateTable {
    pub name: String,
    pub vcpu: u64,
    pub vram: String,
    #[tabled(display("DiskInfo::display_some_vec"))]
    pub disk: Option<Vec<DiskInfo>>,
}

impl VmTemplateTable {
    pub fn from(vm: &VmTemplate) -> Result<Self, VirshleError> {
        let disks: Option<Vec<DiskInfo>> = vm.disk.clone().map(|e| {
            e.iter()
                .map(|e| DiskInfo::from_template(&e).unwrap())
                .collect()
        });
        let table = VmTemplateTable {
            name: vm.name.to_owned(),
            vcpu: vm.vcpu,
            vram: vm.vram.clone(),
            disk: disks,
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
