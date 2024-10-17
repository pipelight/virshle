use super::interface::Ip;
use super::Net;

use pipelight_exec::{Process, Status};
use serde::{Deserialize, Serialize};
use std::fs;
use tabled::{Table, Tabled};

//Database
use crate::database;
use crate::database::connect_db;
use crate::database::entity::{prelude::*, *};
use sea_orm::{
    prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult, IntoActiveModel,
};

// Error Handling
use log::info;
use miette::{Error, IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};
use virshle_error::{LibError, VirshleError, WrapError};

impl Net {
    async fn save_definition(&self) -> Result<(), VirshleError> {
        let res = toml::to_string(&self);
        let value: String = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlSerError(e);
                return Err(err.into());
            }
        };

        // Save Vm to db.
        let record = database::entity::net::ActiveModel {
            uuid: ActiveValue::Set(self.uuid.to_string()),
            name: ActiveValue::Set(self.name.clone()),
            definition: ActiveValue::Set(serde_json::to_value(&self)?),
            ..Default::default()
        };

        let db = connect_db().await?;
        database::prelude::Net::insert(record).exec(&db).await?;

        Ok(())
    }
    pub fn delete(&self) -> Result<(), VirshleError> {
        let cmd = format!("sudo ip link delete {}", self.name,);
        let mut proc = Process::new(&cmd);
        proc.run_piped()?;

        match proc.state.status {
            Some(Status::Failed) => {
                let message = "Couldn't delete network interface";
                if let Some(stderr) = proc.io.stderr {
                    return Err(WrapError::builder()
                        .msg(message)
                        .help("")
                        .origin(Error::msg(stderr))
                        .build()
                        .into());
                }
            }
            _ => {}
        };
        Ok(())
    }

    pub async fn create(&self) -> Result<Self, VirshleError> {
        self.save_definition().await?;
        Ok(self.to_owned())
    }

    pub async fn start(&self) -> Result<Self, VirshleError> {
        let iface = Ip::get_default_interface_name()?;
        let cmd = format!(
            "sudo ip link add link {} name {} type macvtap",
            iface, self.name
        );
        let mut proc = Process::new(&cmd);
        proc.run_piped()?;

        match proc.state.status {
            Some(Status::Failed) => {
                let message = "Couldn't create network interface";
                if let Some(stderr) = proc.io.stderr {
                    return Err(WrapError::builder()
                        .msg(message)
                        .help("")
                        .origin(Error::msg(stderr))
                        .build()
                        .into());
                }
            }
            _ => {}
        };

        let cmd = format!("sudo ip link set {} up", self.name);
        let mut proc = Process::new(&cmd);
        proc.run_piped()?;

        match proc.state.status {
            Some(Status::Failed) => {
                let message = "Couldn't bring network interface up";
                if let Some(stderr) = proc.io.stderr {
                    return Err(WrapError::builder()
                        .msg(message)
                        .help("")
                        .origin(Error::msg(stderr))
                        .build()
                        .into());
                }
            }
            _ => {}
        };

        Ok(self.to_owned())
    }
    /*
     * Start the network.
     */
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn make_net_from_template() -> Result<()> {
        let toml = r#"
        "#;
        let item = Net::from_toml(&toml)?;
        println!("{:#?}", item);
        Ok(())
    }

    #[test]
    fn make_net_from_definition_with_ids() -> Result<()> {
        let toml = r#"
            name = "net_default_b"
            uuid = "ff577daf-07e3-4693-8121-dd1dfb62565e"
            ip = "172.20.0.0/16"
        "#;
        let toml = r#"
            name = "net_default_c"
            uuid = "571c6789-c56f-42df-abf3-b57daec41579"
            ip = "192.168.200.1/24"
        "#;
        let item = Net::from_toml(&toml)?;
        println!("{:#?}", item);
        Ok(())
    }
    // #[tokio::test]
    async fn create_net() -> Result<()> {
        let toml = r#"
            name = "net_macvtap_test"
            uuid = "571c6789-c56f-42df-abf3-b57daec41579"
            ip = "192.168.200.1/24"
        "#;

        let item = Net::from_toml(&toml)?;
        println!("{:#?}", item);

        item.create().await?;
        Ok(())
    }
}
