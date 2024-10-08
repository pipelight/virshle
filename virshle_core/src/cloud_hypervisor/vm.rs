// Cloud Hypervisor
use uuid::Uuid;
use vmm::api::VmInfoResponse;
use vmm::vm::VmState;

use hyper::Request;
use serde::{Deserialize, Serialize};

use pipelight_exec::Process;
use std::io::Write;
use tabled::{
    settings::{object::Columns, Disable, Style},
    Table, Tabled,
};

use super::template;
// Http
use crate::request::Connection;

//Database
use crate::database;
use crate::database::entity::{prelude::*, *};
use sea_orm::{prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult};

use crate::config::MANAGED_DIR;
use crate::display::vm::*;

use crate::database::connect_db;

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};
use virshle_error::{LibError, VirshleError};

use serde_json::{from_slice, Value};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub struct Vm {
    pub name: String,
    pub vcpu: u64,
    #[tabled(display_with = "display_vram")]
    pub vram: u64,
    #[tabled(display_with = "display_state")]
    pub state: Option<VmState>,
    // #[tabled(display_with = "display_ips")]
    // pub ips: Vec<String>,
    pub uuid: Uuid,
}
impl Default for Vm {
    fn default() -> Self {
        Self {
            name: template::random_name().unwrap(),
            vcpu: 1,
            vram: 2,
            state: None,
            // ips: vec![],
            uuid: Uuid::new_v4(),
        }
    }
}

impl Vm {
    /*
     * Retrieve a vm from db
     */
    pub async fn get_by_name(name: &str) -> Result<Self, VirshleError> {
        let db = connect_db().await.unwrap();
        let record = database::prelude::Vm::find()
            .filter(database::entity::vm::Column::Name.eq(name))
            .one(&db)
            .await?;
        if let Some(record) = record {
            let definition_path = MANAGED_DIR.to_owned() + "/vm/" + &record.uuid.to_string();
            Self::from_file(&definition_path)
        } else {
            let message = format!("Could not find a vm with the name: {}", name);
            return Err(LibError::new(&message, "").into());
        }
    }
    pub async fn get_by_uuid(uuid: &Uuid) -> Result<Self, VirshleError> {
        let db = connect_db().await.unwrap();
        let record = database::prelude::Vm::find()
            .filter(database::entity::vm::Column::Uuid.eq(uuid.to_owned()))
            .one(&db)
            .await?;
        if let Some(record) = record {
            let definition_path = MANAGED_DIR.to_owned() + "/vm/" + &record.uuid.to_string();
            Self::from_file(&definition_path)
        } else {
            let message = format!("Could not find a vm with the uuid: {}", uuid);
            return Err(LibError::new(&message, "").into());
        }
    }
    pub fn from_file(file_path: &str) -> Result<Self, VirshleError> {
        let string = fs::read_to_string(file_path)?;
        let res = toml::from_str::<Self>(&string);
        let mut item = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlError(TomlError::new(e, &string));
                return Err(err.into());
            }
        };
        item.update();
        Ok(item)
    }
    pub fn delete(&mut self) {}
    /*
     * Saves the machine definition in the virshle directory at:
     * `/var/lib/virshle/vm/<vm_uuid>`
     * And persists vm "uuid" and "name" into the sqlite database at:
     * `/va/lib/virshle/virshle.sqlite`
     * for fast resource retrieving.
     *
     * ```rust
     * vm.save_definition()?;
     * ```
     *
     * You can find the definition by name and bring the vm
     * back up with:
     * ```rust
     * Vm::get("vm_name")?.set()?;
     * ```
     */
    pub async fn save_definition(&self) -> Result<(), VirshleError> {
        let res = toml::to_string(&self);
        let value: String = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlSerError(e);
                return Err(err.into());
            }
        };

        // Save definition to virshle managed file.
        let definition_path = MANAGED_DIR.to_owned() + "/vm/" + &self.uuid.to_string();
        let toml = toml::to_string(&value);
        let mut file = fs::File::create(definition_path)?;
        file.write_all(value.as_bytes())?;

        // Save Vm to db
        let record = database::entity::vm::ActiveModel {
            uuid: ActiveValue::Set(self.uuid),
            name: ActiveValue::Set(self.name.clone()),
            ..Default::default()
        };
        let db = connect_db().await?;
        database::prelude::Vm::insert(record).exec(&db).await?;

        Ok(())
    }
    /*
     * Bring the virtual machine up.
     */
    pub async fn set(&mut self) -> Result<(), VirshleError> {
        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string();
        let cmd = format!(
            "cloud-hypervisor \
                --api-socket {socket} \
                --kernel /run/cloud-hypervisor/hypervisor-fw \
                --console off \
                --serial tty \
                --disk path=/home/anon/Iso/nixos.efi.qcow2 \
                --cpus boot=2 \
                --memory size=4G",
        );
        let mut proc = Process::new(&cmd);
        proc.run_detached()?;
        self.save_definition().await?;
        Ok(())
    }
    /*
     * Shut the virtual machine down.
     */
    pub async fn shutdown(&mut self) -> Result<(), VirshleError> {
        let socket = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string();
        let endpoint = "/api/v1/vm.info";
        let conn = Connection::open(&socket).await?;

        let response = conn.put::<Vm>(endpoint, None).await?;
        Ok(())
    }
    /*
     * If db is broken, (bypass)
     * Get vm definitions directly from files.
     */
    pub fn get_all_from_file() -> Result<Vec<Vm>, VirshleError> {
        let vm_socket_dir = MANAGED_DIR.to_owned() + "/vm";
        let mut vms: Vec<Vm> = vec![];
        for entry in fs::read_dir(&vm_socket_dir)? {
            let entry = entry?;
            let path = entry.path();
            let mut vm = Self::from_file(path.to_str().unwrap())?;
            vm.update();
            vms.push(vm);
        }
        Ok(vms)
    }
    pub async fn get_all() -> Result<Vec<Vm>, VirshleError> {
        let db = connect_db().await?;
        let records: Vec<database::entity::vm::Model> =
            database::prelude::Vm::find().all(&db).await?;

        let mut vms: Vec<Vm> = vec![];
        for e in records {
            vms.push(Self::get_by_uuid(&e.uuid).await?)
        }
        Ok(vms)
    }
    /*
     * Should be renamed to get_info();
     *
     */
    pub async fn update(&mut self) -> Result<&mut Vm, VirshleError> {
        let socket_addr = MANAGED_DIR.to_owned() + "/socket/" + &self.uuid.to_string();
        let endpoint = "/api/v1/vm.info";

        let conn = Connection::open(&socket_addr).await?;
        let response = conn.get(endpoint).await?;

        let data = response.to_string().await?;
        println!("{:#?}", data);
        let data: VmInfoResponse = serde_json::from_str(&data)?;

        self.state = Some(data.state);
        Ok(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    // #[tokio::test]
    async fn fetch_info() -> Result<()> {
        Vm::get_by_name("default_xs").await?.update().await?;
        Ok(())
    }

    // #[tokio::test]
    async fn fetch_vms() -> Result<()> {
        let items = Vm::get_all().await?;
        println!("{:#?}", items);
        Ok(())
    }

    // #[tokio::test]
    async fn set_vm_from_file() -> Result<()> {
        // Get file
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../templates/ch/vm/xs.toml");
        let path = path.display().to_string();

        let mut item = Vm::from_file(&path)?;
        item.set().await?;
        Ok(())
    }

    #[tokio::test]
    async fn set_vm() -> Result<()> {
        let mut item = Vm::default();
        item.set().await?;
        Ok(())
    }
    // #[tokio::test]
    async fn delete_vm() -> Result<()> {
        let mut item = Vm::default();
        item.shutdown().await?;
        Ok(())
    }
}
