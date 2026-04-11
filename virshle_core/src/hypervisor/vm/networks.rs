use crate::config::{Config, DhcpType, NetType, VmNet};
use crate::hypervisor::{Vm, VmTable};
use crate::network::{
    dhcp::{FakeDhcp, KeaDhcp, Lease},
    ip,
    ovs::{OvsBridge, OvsPort},
};

use std::fs;
use std::net::IpAddr;
use std::path::Path;

// Error Handling
use miette::Result;
use tracing::trace;
use virshle_error::{LibError, VirshleError};

impl Vm {
    pub fn networks(&self) -> VmNetMethods<'_> {
        VmNetMethods { vm: self }
    }
}
pub struct VmNetMethods<'a> {
    pub vm: &'a Vm,
}

impl VmNetMethods<'_> {
    /// Return vm ips,
    /// or an empty vec if nothing found.
    pub async fn ips(&self) -> Result<Vec<IpAddr>, VirshleError> {
        let mut ips = vec![];
        if let Some(leases) = self.leases().get_all().await.ok() {
            ips = leases.iter().map(|e| e.address).collect();
        }
        Ok(ips)
    }
    /// Create network <name> on host (and ovs configuration).
    #[tracing::instrument(skip_all)]
    pub fn create_one(&self, name: &str) -> Result<(), VirshleError> {
        if let Some(e) = self.vm.net.clone() {
            let nets: Vec<VmNet> = e.into_iter().filter(|e| e.name == name).collect();
            let net = nets.first();
            match net {
                Some(v) => {
                    self._create(v)?;
                }
                None => {}
            };
        }
        Ok(())
    }
    /// Create all networks associated to Vm on host (and ovs configuration).
    #[tracing::instrument(skip_all)]
    pub fn create_all(&self) -> Result<(), VirshleError> {
        trace!("creating networks for vm {:#?}", self.vm.name);
        if let Some(networks) = &self.vm.net {
            for net in networks {
                // Clean up
                self._delete(&net)?;
                self._create(&net)?;
            }
        }
        Ok(())
    }
    /// Create a network on host (and ovs configuration).
    fn _create(&self, net: &VmNet) -> Result<(), VirshleError> {
        // This results in "machin_name-network_name".
        let port_name = format!("vm-{}--{}", self.vm.name, net.name);
        match &net._type {
            // Vhost type does not work on when bridged to ovs-bridge of type "system",
            // the bridge must be of type "netdev".
            NetType::Vhost(v) => {
                let socket_path = self.vm.get_net_socket(&net)?;
                OvsBridge::get_vm_switch()?.create_dpdk_port(&port_name, &socket_path)?;
            }
            // Tap do not work on ovs-bridge of type "netdev",
            // the bridge must be of type "system".
            NetType::Tap(v) => {
                // Create tap device
                ip::tap::create(&port_name)?;
                ip::up(&port_name)?;

                // Link to ovs bridge
                let vmbr = OvsBridge::get_vm_switch()?;
                // Silently try to delete old port if any.
                match OvsPort::get_by_name(&port_name) {
                    Ok(v) => {
                        v.delete()?;
                    }
                    Err(_) => {}
                };
                vmbr.create_tap_port(&port_name)?;
            }
            // MacVTap do not work on ovs-bridge of type "netdev",
            // the bridge must be of type "system".
            NetType::MacVTap(v) => {
                // Create macvtap device
                ip::macvtap::create(&port_name)?;
                ip::up(&port_name)?;
            }
        };
        Ok(())
    }
    /// Remove network <name> from host (and ovs configuration).
    pub fn delete_one(&self, name: &str) -> Result<(), VirshleError> {
        if let Some(e) = self.vm.net.clone() {
            let nets: Vec<VmNet> = e.into_iter().filter(|e| e.name == name).collect();
            let net = nets.first();
            match net {
                Some(v) => {
                    self._delete(v)?;
                }
                None => {}
            };
        }
        Ok(())
    }
    /// Remove all networks associated to Vm from host (and ovs configuration).
    pub fn delete_all(&self) -> Result<(), VirshleError> {
        if let Some(e) = &self.vm.net {
            for net in e {
                self._delete(&net)?;
            }
        }
        Ok(())
    }
    /// Remove a network from host (and ovs configuration).
    /// WARNING: Silently fail (due to ".ok()").
    fn _delete(&self, net: &VmNet) -> Result<(), VirshleError> {
        // This results in "machin_name-network_name".
        let port_name = format!("vm-{}--{}", self.vm.name, net.name);

        // Ovs: try to delete the port and silently fail.
        if let Some(port) = OvsBridge::get_vm_switch()?.get_port(&port_name).ok() {
            port.delete().ok();
        }

        match &net._type {
            NetType::Tap(_) | NetType::MacVTap(_) => {
                // Use the ip command to delete interfaces.
                ip::tap::delete(&port_name).ok();
            }
            NetType::Vhost(_) => {
                // Delete existing socket if any because
                // cloud-hypervisor will attempt to create a new socket or fail.
                let socket_path = self.vm.get_net_socket(&net)?;
                let path = Path::new(&socket_path);
                if path.exists() {
                    fs::remove_file(&socket_path).ok();
                }
            }
        };
        Ok(())
    }
}

impl VmNetMethods<'_> {
    pub fn leases(&self) -> VmLeaseMethods {
        VmLeaseMethods { vm: self.vm }
    }
}
pub struct VmLeaseMethods<'a> {
    pub vm: &'a Vm,
}
impl VmLeaseMethods<'_> {
    /// Delete Vm dhcp ipv4 and ipv6 leases .
    pub async fn delete_all(&self) -> Result<(), VirshleError> {
        match Config::get()?.dhcp {
            Some(DhcpType::Fake(fake_dhcp_config)) => {
                if let Some(id) = self.vm.id {
                    FakeDhcp::delete_leases(id.try_into().unwrap()).await?;
                }
            }
            Some(DhcpType::Kea(kea_dhcp_config)) => {
                let mut cli = KeaDhcp::builder().config(kea_dhcp_config).build().await?;
                let leases = cli
                    .lease()
                    .get()
                    .many()
                    .inet4(true)
                    .inet6(true)
                    .vm(self.vm.clone())
                    .exec()
                    .await?;
                cli.lease()
                    .delete()
                    .many()
                    .leases(leases)
                    .inet4(true)
                    .inet6(true)
                    .exec()
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
    /// Return vm leases,
    /// or error out if nothing found
    pub async fn get_all(&self) -> Result<Vec<Lease>, VirshleError> {
        let mut leases: Vec<Lease> = vec![];
        let config = Config::get()?;
        match config.dhcp {
            Some(DhcpType::Kea(kea_dhcp_config)) => {
                let mut cli = KeaDhcp::builder().config(kea_dhcp_config).build().await?;
                leases = cli
                    .lease()
                    .get()
                    .many()
                    .inet4(true)
                    .inet6(true)
                    .vm(self.vm.clone())
                    .exec()
                    .await?;
            }
            _ => {}
        };
        if leases.is_empty() {
            let message = format!("Couldn't find a lease for vm: {}", self.vm.name);
            let help = "Are you sure the VM has already requested an address from kea-dhcp";
            let err = LibError::builder().msg(&message).help(&help).build();
            Err(err.into())
        } else {
            Ok(leases)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn fetch_one_ips() -> Result<()> {
        let items = Vm::database().await?.many().get().await?;
        if let Some(vm) = items.first() {
            let vm = Vm::database().await?.one().uuid(vm.uuid).get().await?;
            let leases = vm.networks().leases().get_all().await?;
            println!("{:#?}", leases);
        }
        Ok(())
    }
}
