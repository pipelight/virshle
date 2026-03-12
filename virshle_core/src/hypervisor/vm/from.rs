use crate::hypervisor::{disk::utils, Disk, Vm};

// Database
use crate::database::*;

// Pretty print
use bat::PrettyPrinter;
use crossterm::{style::Stylize, terminal::size};
use log::{log_enabled, Level};

// Filesystem manipulation
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use std::convert::Into;

use uuid::Uuid;

use virshle_network::connection::{Connection, SshConnection, TcpConnection, UnixConnection, Uri};

// Global configuration
use crate::config::MANAGED_DIR;

// Error Handling
use miette::Result;
use tracing::trace;
use virshle_error::{CastError, TomlError, VirshleError, WrapError};

impl TryInto<Vm> for &vm::Model {
    type Error = VirshleError;
    fn try_into(self) -> Result<Vm, Self::Error> {
        let definition: Vm = serde_json::from_value(self.definition.clone())?;
        let vm = Vm {
            uuid: Uuid::parse_str(&self.uuid)?,
            name: self.name.clone(),
            ..definition
        };
        Ok(vm)
    }
}
impl TryInto<Connection> for Vm {
    type Error = VirshleError;
    fn try_into(self) -> Result<Connection, Self::Error> {
        (&self).try_into()
    }
}
impl TryInto<Connection> for &mut Vm {
    type Error = VirshleError;
    fn try_into(self) -> Result<Connection, Self::Error> {
        (&*self).try_into()
    }
}
impl TryInto<Connection> for &Vm {
    type Error = VirshleError;
    fn try_into(self) -> Result<Connection, Self::Error> {
        let uri = self.vmm().get_socket_uri().unwrap();
        let conn = match Uri::new(&uri).unwrap() {
            Uri::SshUri(v) => Connection::SshConnection(SshConnection {
                uri: v,
                ssh_handle: None,
            }),
            Uri::LocalUri(v) => Connection::UnixConnection(UnixConnection { uri: v }),
            Uri::TcpUri(v) => Connection::TcpConnection(TcpConnection { uri: v }),
        };
        trace!("created connection for vm: {}", self.uuid);
        Ok(conn)
    }
}

impl Vm {
    /*
     * Create a vm from a file containing a Toml definition.
     */
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
                let err = WrapError::builder()
                    .msg("Couldn't convert toml string to a valid vm")
                    .help("")
                    .origin(err.into())
                    .build();
                return Err(err.into());
            }
        };
        Ok(item)
    }
    pub fn print_to_toml(&self) -> Result<String, VirshleError> {
        let string: String = toml::to_string(self).map_err(CastError::from)?;
        if log_enabled!(Level::Warn) {
            let (cols, _) = size()?;
            let divider = "-".repeat((cols / 3).into());
            println!("{}", format!("{divider}toml{divider}").green());
            PrettyPrinter::new()
                .input_from_bytes(string.as_bytes())
                .language("toml")
                .print()?;
            println!("{}", format!("{divider}----{divider}").green());
            println!("");
        }
        Ok(string)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::VmTemplate;
    use miette::IntoDiagnostic;

    #[test]
    fn display_vm_to_toml() -> Result<()> {
        let vm = Vm::default();
        let string = vm.print_to_toml()?;
        println!("");
        PrettyPrinter::new()
            .input_from_bytes(string.as_bytes())
            .language("toml")
            .print()
            .into_diagnostic()?;
        Ok(())
    }
    #[test]
    fn make_vm_template_from_toml() -> Result<()> {
        let toml = r#"
            name = "my_template"

            vcpu = 1
            vram = "2GiB"

            [[disk]]
            name = "os"
            path = "~/tmp/disk/template.iso"

            [[net]]
            name = "main"
            [net.type.tap]
        "#;
        let item = VmTemplate::from_toml(&toml)?;
        println!("{:#?}", item);
        Ok(())
    }
    #[test]
    fn make_vm_from_toml() -> Result<()> {
        let toml = r#"
            name = "vm-my-test"
            uuid = "b30458d1-7c7f-4d06-acc2-159e43892e87"

            vcpu = 1
            vram = "2GiB"

            [[disk]]
            name = "os"
            path = "~/tmp/disk/uuid.iso"

            [[net]]
            name = "main"
            [net.type.tap]
            "#;
        let item = Vm::from_toml(&toml)?;
        println!("{:#?}", item);
        Ok(())
    }
}
