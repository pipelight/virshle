pub mod api;
pub mod types;

use std::path::Path;

// Process management
use pipelight_exec::{Finder, Process, Status};
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

use crate::config::VmTemplate;
use crate::hypervisor::Vm;

pub use types::{VmConfig, VmInfoResponse, VmRemoveDeviceData, VmState};

// Error Handling
use miette::Result;
use tracing::info;
use virshle_error::VirshleError;

impl Vm {
    pub fn vmm(&self) -> VmmMethods<'_> {
        VmmMethods { vm: self }
    }
}
pub struct VmmMethods<'a> {
    vm: &'a Vm,
}
// Vmm API
impl VmmMethods<'_> {
    /// Remove running vm hypervisor process if any
    /// and assiociated socket.
    pub fn kill_process(&self) -> Result<(), VirshleError> {
        let finder = Finder::new()
            .seed("cloud-hypervisor")
            .seed(&self.vm.uuid.to_string())
            .search_no_parents()?;

        #[cfg(debug_assertions)]
        if let Some(matches) = finder.matches {
            for _match in matches {
                if let Some(pid) = _match.pid {
                    Process::new().stdin(&format!("sudo kill -9 {pid}")).run()?;
                }
            }
        }
        #[cfg(not(debug_assertions))]
        finder.kill()?;

        let socket = &self.get_socket()?;
        let path = Path::new(&socket);
        if path.exists() {
            #[cfg(debug_assertions)]
            Process::new()
                .stdin(&format!("sudo rm {}", &socket))
                .run()?;
            #[cfg(not(debug_assertions))]
            fs::remove_file(&socket)?;
        }

        let vsock = &self.vm.get_vsocket()?;
        let path = Path::new(&vsock);
        if path.exists() {
            #[cfg(debug_assertions)]
            Process::new().stdin(&format!("sudo rm {}", &vsock)).run()?;

            #[cfg(not(debug_assertions))]
            fs::remove_file(&vsock)?;
        }

        Ok(())
    }
    /// Start or Restart a VMM.
    pub async fn start(&self, attach: Option<bool>) -> Result<(), VirshleError> {
        // Safeguard: remove old process and artifacts
        self.kill_process()?;

        #[cfg(debug_assertions)]
        let mut cmd = format!("cloud-hypervisor");
        #[cfg(not(debug_assertions))]
        let mut cmd = format!("cloud-hypervisor");

        // If we can't establish connection to socket,
        // this means cloud-hypervisor is dead.
        // So we start a new viable process.

        if self.api()?.ping().await.is_err() {
            match attach {
                Some(true) => {
                    cmd = format!(
                        "kitty \
                            --title ttyS0@vm-{} \
                            --hold sh -c \"{} --api-socket {}\"",
                        &self.vm.name,
                        cmd,
                        &self.get_socket()?
                    );
                    Process::new()
                        .stdin(&cmd)
                        .term()
                        .background()
                        .detach()
                        .run()?;
                    info!("launching: {:#?}", &cmd);
                }
                _ => {
                    cmd = format!("{} --api-socket {}", &cmd, &self.get_socket()?);
                    Process::new()
                        .stdin(&cmd)
                        .orphan()
                        .background()
                        .detach()
                        .run()?;
                    info!("launching: {:#?}", &cmd);
                }
            };

            // Wait until socket is created
            let socket = &self.get_socket()?;
            let path = Path::new(socket);
            while !path.exists() {
                tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
            }

            // Set loose permission on cloud-hypervisor socket.
            #[cfg(not(debug_assertions))]
            {
                let mut perms = fs::metadata(&path)?.permissions();
                perms.set_mode(0o774);
                fs::set_permissions(&path, perms)?;
            }
            #[cfg(debug_assertions)]
            Process::new()
                .stdin(&format!("sudo chmod 774 {}", &socket))
                .run()?;
        }
        Ok(())
    }
}
