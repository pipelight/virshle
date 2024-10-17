mod interface;
mod to;

use interface::Ip;

use super::rand::random_place;
use pipelight_exec::{Process, Status};
use serde::{Deserialize, Serialize};
use std::fs;
use tabled::{Table, Tabled};

// Error Handling
use log::info;
use miette::{Error, IntoDiagnostic, Result};
use pipelight_error::{CastError, TomlError};
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct NetTemplate {
    name: Option<String>,
    // CIDR notation ip/subnet_mask
    ip: Option<String>,
    // autostart net on host boot
    enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Net {
    name: String,
    // CIDR notation ip/subnet_mask
    ip: String,
    // autostart net on host boot
    enabled: bool,
}

impl Default for Net {
    fn default() -> Self {
        Self {
            name: random_place().unwrap(),
            ip: "192.168.200.1/24".to_owned(),
            enabled: true,
        }
    }
}

impl From<&NetTemplate> for Net {
    fn from(e: &NetTemplate) -> Self {
        let mut net = Self {
            ..Default::default()
        };
        if let Some(name) = &e.name {
            net.name = name.to_owned();
        }
        if let Some(ip) = &e.ip {
            net.ip = ip.to_owned();
        }
        net
    }
}
impl Net {
    pub fn delete(&self) -> Result<(), VirshleError> {
        Ok(())
    }
    pub fn create(&self) -> Result<Self, VirshleError> {
        let iface = Ip::get_default_interface_name()?;
        let cmd = format!(
            "sudo ip link add link {} name {} type macvtap",
            iface, self.name
        );
        let mut proc = Process::new(&cmd);
        proc.run_piped()?;

        match proc.state.status {
            Some(Status::Failed) => {
                let message = "Couldn't create network";
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
    pub fn start(&self) -> Result<Self, VirshleError> {
        let cmd = format!("ip link set {} up", self.name);
        let proc = Process::new(&cmd);
        Ok(self.to_owned())
    }
}
impl Net {
    pub fn from_file(file_path: &str) -> Result<Self, VirshleError> {
        let string = fs::read_to_string(file_path)?;
        Self::from_toml(&string)
    }
    pub fn from_toml(string: &str) -> Result<Self, VirshleError> {
        let res = toml::from_str::<NetTemplate>(&string);

        let item: NetTemplate = match res {
            Ok(res) => res,
            Err(e) => {
                let err = CastError::TomlError(TomlError::new(e, &string));
                return Err(err.into());
            }
        };
        let item = Net::from(&item);
        Ok(item)
    }
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
    #[test]
    fn create_net() -> Result<()> {
        let toml = r#"
            name = "net_macvtap_test"
            uuid = "571c6789-c56f-42df-abf3-b57daec41579"
            ip = "192.168.200.1/24"
        "#;

        let item = Net::from_toml(&toml)?;
        println!("{:#?}", item);

        item.create()?;
        Ok(())
    }
}
