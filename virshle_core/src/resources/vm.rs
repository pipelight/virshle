use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tabled::Tabled;

// Error Handling
use crate::error::{VirshleError, VirtError, WrapError};
use log::trace;
use miette::{IntoDiagnostic, Result};

// libvirt
use super::connect;
use crate::convert;
use convert_case::{Case, Casing};
use human_bytes::human_bytes;
use strum::EnumIter;
use virt::domain::Domain;

use once_cell::sync::Lazy;

static NVirConnectListAllNetworksFlags: u32 = 5;

fn display_option(state: &Option<State>) -> String {
    match state {
        Some(s) => format!("{}", s),
        None => format!(""),
    }
}
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

#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub struct Vm {
    pub name: String,
    pub id: u32,
    pub vcpu: u64,
    pub vram: String,
    pub state: State,
}
impl Vm {
    fn from(e: &Domain) -> Result<Vm, VirshleError> {
        let res = Vm {
            id: e.get_id().unwrap(),
            name: e.get_name()?,
            state: State::from(e.is_active()? as u32),
            vcpu: e.get_max_vcpus()?,
            vram: human_bytes(e.get_max_memory()? as f64),
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

        for flag in 0..NVirConnectListAllNetworksFlags {
            let items = conn.list_all_domains(flag)?;
            for item in items.clone() {
                let vm = Vm::from(&item)?;
                let name = vm.clone().name;
                if !map.contains_key(&name) {
                    map.insert(name, vm);
                }
            }
        }
        let list: Vec<Vm> = map.into_values().collect();
        Ok(list)
    }
    pub fn set(path: &str) -> Result<(), VirshleError> {
        let toml = fs::read_to_string(path)?;
        let xml = convert::from_toml_to_xml(&toml)?;
        Self::set_xml(&xml)?;
        Ok(())
    }
    pub fn set_xml(xml: &str) -> Result<(), VirshleError> {
        let conn = connect()?;
        let res = Domain::create_xml(&conn, &xml, 0);
        match res {
            Ok(res) => Ok(()),
            Err(e) => Err(VirtError::new("The Vm could not be created", "", e).into()),
        }
    }
    pub fn delete(name: &str) -> Result<(), VirshleError> {
        // Guard
        Self::get(name)?;

        let conn = connect()?;
        let item = Domain::lookup_by_name(&conn, name)?;
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
        path.push("../templates/vm/base.toml");
        let path = path.display().to_string();

        let items = Vm::set(&path);
        println!("{:#?}", items);
        Ok(())
    }
}
