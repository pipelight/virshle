use crate::display;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::fs;
// Error Handling
use crate::error::{VirshleError, WrapError};
use log::trace;
use miette::{IntoDiagnostic, Result};
// libvirt
use virt::connect::Connect;
use virt::domain::Domain;

pub fn connect() -> Result<Connect, VirshleError> {
    // let conn = Connect::open(Some("test:///default")).into_diagnostic()?;
    let res = Connect::open(Some("qemu:///system"))?;
    Ok(res)
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Vm {
    pub name: String,
    pub id: u32,
    pub vcpu: u64,
    pub vram: u64,
    pub state: Option<State>,
}
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum State {
    Saved,
    Running,
    Paused,
    Defined,
    Undefined,
}

impl Vm {
    pub fn get(id: u32) -> Result<Self, VirshleError> {
        let conn = connect()?;
        let domain = Domain::lookup_by_id(&conn, id)?;
        let vm = Vm {
            id,
            name: domain.get_name()?,
            vcpu: domain.get_max_vcpus()?,
            vram: domain.get_max_memory()?,
            state: None,
        };
        Ok(vm)
    }
    pub fn get_all() -> Result<Vec<Self>> {
        let conn = connect()?;
        let domain_id_list = conn.list_domains().into_diagnostic()?;
        let mut vms = vec![];
        for id in domain_id_list {
            vms.push(Vm::get(id)?);
        }
        Ok(vms)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_connect() -> Result<()> {
        let res = connect();
        assert!(res.is_ok());
        Ok(())
    }

    #[test]
    fn fetch_node_info() -> Result<()> {
        let res = connect()?;
        let info = res.get_node_info().into_diagnostic()?;
        println!("{:#?}", info);
        Ok(())
    }

    #[test]
    fn fetch_domains() -> Result<()> {
        let conn = connect()?;
        let domain_id_list = conn.list_domains().into_diagnostic()?;
        for id in domain_id_list {
            let vm = Vm::get(id);
            println!("{:#?}", vm);
        }
        Ok(())
    }
}
