use serde::{Deserialize, Serialize};
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
use strum::EnumIter;
use virt::domain::Domain;

fn display_option(state: &Option<State>) -> String {
    match state {
        Some(s) => format!("{}", s),
        None => format!(""),
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub struct Vm {
    pub name: String,
    pub id: u32,
    pub vcpu: u64,
    pub vram: u64,
    // #[tabled(display_with = "display_option")]
    pub state: State,
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
impl Vm {
    pub fn get(id: u32) -> Result<Self, VirshleError> {
        let conn = connect()?;
        let domain = Domain::lookup_by_id(&conn, id)?;

        let (state, _) = domain.get_state()?;
        let vm = Vm {
            id,
            name: domain.get_name()?,
            vcpu: domain.get_max_vcpus()?,
            vram: domain.get_max_memory()?,
            state: State::from(state),
            ..Default::default()
        };
        Ok(vm)
    }
    pub fn get_all() -> Result<Vec<Self>, VirshleError> {
        let conn = connect()?;
        let ids = conn.list_domains()?;
        let mut list = vec![];
        for id in ids {
            list.push(Vm::get(id)?);
        }
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
