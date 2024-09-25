// Convert
use crate::convert::{from_toml, from_toml_to_xml, to_xml};
use serde_json::Value;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tabled::Tabled;
use uuid::Uuid;

// libvirt
use super::connect;
use crate::convert;
use convert_case::{Case, Casing};
use human_bytes::human_bytes;
use strum::EnumIter;
use virt::domain::{Domain, Interface};

// Error Handling
use crate::error::{VirshleError, VirtError, WrapError};
use log::{info, trace};
use miette::{IntoDiagnostic, Result};

static NVirConnectListAllDomainsFlags: u32 = 15;

#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq, EnumIter)]
pub enum State {
    #[default]
    NoState = 0,
    Running = 1,
    Blocked = 2,
    Paused = 3,
    ShutDown = 4,
    ShutOff = 5,
    Crashed = 6,
    PmSuspended = 7,
    Last = 8,
}
impl From<u32> for State {
    fn from(value: u32) -> Self {
        match value {
            0 => State::NoState,
            1 => State::Running,
            2 => State::Blocked,
            3 => State::Paused,
            4 => State::ShutDown,
            5 => State::ShutOff,
            6 => State::Crashed,
            7 => State::PmSuspended,
            8 => State::Last,
            _ => State::NoState,
        }
    }
}

fn display_vram(vram: &u64) -> String {
    let res = human_bytes((vram * 1024) as f64);
    format!("{}", res)
}
fn display_id(id: &Option<u32>) -> String {
    match id {
        Some(x) => x.to_string().to_owned(),
        None => "_".to_owned(),
    }
}
fn display_ips(ips: &Vec<String>) -> String {
    let res = ips.join("\n");
    format!("{}\n", res)
}
#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub struct Vm {
    pub name: String,
    #[tabled(display_with = "display_id")]
    pub id: Option<u32>,
    pub vcpu: u64,
    #[tabled(display_with = "display_vram")]
    pub vram: u64,
    pub state: State,
    #[tabled(display_with = "display_ips")]
    pub ips: Vec<String>,
    pub uuid: Uuid,
}

impl Vm {
    fn from(e: &Domain) -> Result<Vm, VirshleError> {
        let res = Vm {
            id: Self::get_id(e)?,
            name: e.get_name()?,
            state: State::from(e.is_active()? as u32),
            vcpu: Self::get_cpus(e)?,
            vram: e.get_max_memory()?,
            uuid: e.get_uuid()?,
            ips: Self::get_ips(e)?,
            ..Default::default()
        };
        Ok(res)
    }
    pub fn get(name: &str) -> Result<Self, VirshleError> {
        let conn = connect()?;
        let res = Domain::lookup_by_name(&conn, name);
        match res {
            Ok(e) => {
                let item = Vm::from(&e)?;
                Ok(item)
            }
            Err(e) => Err(VirtError::new(
                &format!("No vm with name {:?}", name),
                "Maybe you made a typo",
                e,
            )
            .into()),
        }
    }
    pub fn get_all() -> Result<Vec<Self>, VirshleError> {
        let conn = connect()?;
        let mut map: HashMap<String, Vm> = HashMap::new();

        for flag in 0..NVirConnectListAllDomainsFlags {
            let items = conn.list_all_domains(flag)?;
            for item in items.clone() {
                let vm = Vm::from(&item)?;
                let name = vm.name.clone();
                if !map.contains_key(&name) {
                    map.insert(name, vm);
                }
            }
        }
        let mut list: Vec<Vm> = map.into_values().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(list)
    }
    pub fn get_id(e: &Domain) -> Result<Option<u32>, VirshleError> {
        // Guard
        if e.is_active()? {
            Ok(e.get_id())
        } else {
            Ok(None)
        }
    }
    /*
     * If the domain is running get maximum vcpu allowed,
     *  else return vcpu from domain definition.
     */
    pub fn get_cpus(e: &Domain) -> Result<u64, VirshleError> {
        if e.is_active()? {
            Ok(e.get_max_vcpus()?)
        } else {
            let vcpus = e.get_info()?.nr_virt_cpu as u64;
            Ok(vcpus)
        }
    }
    /*
     * https://libvirt.org/html/libvirt-libvirt-domain.html#virDomainInterfaceAddressesSource
     */
    pub fn get_ips(e: &Domain) -> Result<Vec<String>, VirshleError> {
        // Guard
        if e.is_active()? {
            let res = e.interface_addresses(0, 0);
            match res {
                Ok(res) => {
                    let ips: Vec<String> = res
                        .iter()
                        .map(|e| {
                            e.addrs
                                .iter()
                                .map(|a| a.addr.clone())
                                .collect::<Vec<String>>()
                        })
                        .flatten()
                        .collect();
                    Ok(ips)
                }
                Err(e) => Err(VirtError::new("Couldn't get the vm ips", "", e).into()),
            }
        } else {
            Ok(vec![])
        }
    }
    pub fn set_w_uuid(path: &str, uuid: &Uuid) -> Result<(), VirshleError> {
        let toml = fs::read_to_string(path)?;

        // Hydrate template with uuid and volume ids,...
        let mut value = from_toml(&toml)?;
        value["domain"]["uuid"] = Value::String(uuid.to_string());
        info!("\n{}", toml);

        let xml = to_xml(&value)?;
        info!("\n{}", xml);

        Self::set_xml(&xml)?;
        Ok(())
    }
    pub fn set(path: &str) -> Result<(), VirshleError> {
        let toml = fs::read_to_string(path)?;
        let xml = convert::from_toml_to_xml(&toml)?;
        Self::set_xml(&xml)?;
        Ok(())
    }
    pub fn set_xml(xml: &str) -> Result<(), VirshleError> {
        let conn = connect()?;
        let res = Domain::define_xml(&conn, &xml);
        match res {
            Ok(res) => Ok(()),
            Err(e) => Err(VirtError::new("The Vm could not be created", "", e).into()),
        }
    }
}

// Methods
impl Vm {
    pub fn shutdown(&self) -> Result<(), VirshleError> {
        // Guard
        Self::get(&self.name)?;

        let conn = connect()?;
        let item = Domain::lookup_by_name(&conn, &self.name)?;
        let res = item.shutdown();
        match res {
            Ok(res) => Ok(()),
            Err(e) => Err(VirtError::new("Libvirt could not shutdown the vm", "", e).into()),
        }
    }
    pub fn reboot(&self) -> Result<(), VirshleError> {
        // Guard
        Self::get(&self.name)?;

        let conn = connect()?;
        let item = Domain::lookup_by_name(&conn, &self.name)?;
        let res = item.reboot(0);
        match res {
            Ok(res) => Ok(()),
            Err(e) => Err(VirtError::new("Libvirt could not reboot the vm", "", e).into()),
        }
    }
    pub fn delete(&self) -> Result<(), VirshleError> {
        // Guard
        Self::get(&self.name)?;

        let conn = connect()?;
        let item = Domain::lookup_by_name(&conn, &self.name)?;
        let res = item.destroy();
        match res {
            Ok(res) => Ok(()),
            Err(e) => Err(VirtError::new("The vm could not be destroyed", "", e).into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn fetch_domains() -> Result<()> {
        let items = Vm::get_all();
        println!("{:#?}", items);
        Ok(())
    }

    #[test]
    fn create_domain() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../templates/vm/default_qcow2.toml");
        let path = path.display().to_string();

        let items = Vm::set(&path);
        println!("{:#?}", items);
        Ok(())
    }

    // #[test]
    fn delete_domain() -> Result<()> {
        Vm::get("vm-nixos")?.delete()?;
        Ok(())
    }
}
