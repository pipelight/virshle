use crate::network::dhcp::{KeaDhcp, Lease, kea::{types::RestResponse, RawLease}};
use crate::network::utils::{uuid_to_mac, uuid_to_duid};
use crate::hypervisor::{Vm, VmTable};

use bon::{bon, builder};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

use std::net::IpAddr;

use virshle_network::{
    connection::{Connection, TcpConnection},
    http::{Rest, RestClient},
};

// Error handling
use miette::Result;
use virshle_error::{LibError, VirshleError, WrapError};

#[serde_with::skip_serializing_none]
#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct KeaCommand {
    command: String,
    service: Vec<String>,
    arguments: Option<HashMap<String, String>>,
}
#[serde_with::skip_serializing_none]
#[derive(Default, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct KeaBulkCommand {
    command: String,
    service: Vec<String>,
    arguments: Option<HashMap<String, Vec<HashMap<String, String>>>>,
}

impl KeaDhcp {
    pub fn ip(&mut self) -> IpMethods<'_> {
        IpMethods { api: self }
    }
}
pub struct IpMethods<'a> {
    api: &'a mut KeaDhcp,
}

pub struct IpGetterMethods<'a> {
    api: &'a mut KeaDhcp,
}
impl IpMethods<'_> {
    pub fn get(&mut self) -> IpGetterMethods<'_> {
        IpGetterMethods { api: self.api }
    }
}

#[bon]
impl IpGetterMethods<'_> {
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn many(
        &mut self,
        inet6: bool, 
        inet4: bool,
        vm: Option<Vm>,
    ) -> Result<Vec<IpAddr>, VirshleError> {
        let leases = self.api.lease().get().many().maybe_vm(vm).inet6(inet6).inet4(inet4).exec().await?;
        let res: Vec<IpAddr> = leases.into_iter().map(|e| e.address).collect();
        Ok(res)
    }
}

impl KeaDhcp {
    pub fn lease(&mut self) -> LeaseMethods<'_> {
        LeaseMethods { api: self }
    }
    // UTILS
    //
    // Can't get leases by hostname without suffuring because of inconsistency in hostname storage between kea dhcp6 and kea dhcp4 servers.
    // (ipv6 -> with a dot "." AND ipv4 -> without dot)
    //
    // So we use the hardware address (MAC address) 
    // computed based on VM uuid..
    pub fn vm_to_args(&self, vm: &VmTable) -> Vec<(String, String)> {
        let hwaddr = uuid_to_mac(&vm.uuid).to_string();
        let args: Vec<(String, String)> = vec![
            ("hw-address".to_owned(), hwaddr)
            // ("hostname".to_owned(), hostname)
        ];
        args
    }
}
pub struct LeaseMethods<'a> {
    api: &'a mut KeaDhcp,
}

#[bon]
impl LeaseMethods<'_> {
    /// Clean all leases.
    /// Remove lease if associated vm doesn't exist (in virshle database).
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn clean(
        &mut self,
        inet6: bool, 
        inet4: bool,
    ) -> Result<(), VirshleError> {

        // Get vms
        let vms: Vec<String> = Vm::database()
            .await?
            .many()
            .get()
            .await?
            .iter()
            .map(|e| e.name.clone())
            .collect();

        // Get leases
        let mut leases = self.get().many().inet6(true).inet4(true).exec().await?;

        // Remove leases if no corresponding vm name
        leases = leases
            .into_iter()
            .filter(|e|
                !vms.contains(
                    &e.vm_name().suffix("vm").extract().unwrap()
                )
            )
            .map(|e| e.to_owned())
            .collect();

        self.delete()
            .many()
            .leases(leases)
            .inet6(inet6)
            .inet4(inet4)
            .exec()
            .await?;

        Ok(())
    }
}

impl LeaseMethods<'_> {
    pub fn get(&mut self) -> LeaseGetterMethods<'_> {
        LeaseGetterMethods { api: self.api }
    }
}
pub struct LeaseGetterMethods<'a> {
    api: &'a mut KeaDhcp,
}
#[bon]
impl LeaseGetterMethods<'_> {
    // Default behavior: get all ipv6 leases.
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn many(
        &mut self,
        inet6: bool, 
        inet4: bool,
        vm: Option<Vm>,
    ) -> Result<Vec<Lease>, VirshleError> {
        // Default command: get all ipv6 leases.
        let default_cmd = KeaCommand {
            command: "lease6-get-all".to_owned(),
            service: vec!["dhcp6".to_owned()],
            arguments: None,
        };

        let mut cmds: Vec<KeaCommand> = vec![];
        if inet4 {
            let mut cmd = KeaCommand {
                command: "lease4-get-all".to_owned(),
                service: vec!["dhcp4".to_owned()],
                ..default_cmd.clone()
            };
            // Get leases for a specified Vm.
            if let Some(ref vm) = vm {

                // DEV WARNING: 
                // Inconsistency in hostname storage between kea dhcp6 and dhcp4 :
                // ipv6 -> with a trailing dot "vm.random." 
                // ipv4 -> without trailing dot "vm.random"
                // let hostname = vm.name.clone();
                // let args: HashMap<String, String> = HashMap::from([
                //     ("hostname".to_owned(), hostname)
                // ]);
                // cmd = KeaCommand {
                //     command: cmd.command.replace("all", "by-hostname"),
                //     arguments: Some(args),
                //     ..cmd.clone()
                // };
                
                // Alternative:
                let hwaddr = uuid_to_mac(&vm.uuid).to_string();
                let args: HashMap<String, String> = HashMap::from([
                    ("hw-address".to_owned(), hwaddr)
                ]);
                cmd = KeaCommand {
                    command: cmd.command.replace("all", "by-hw-address"),
                    arguments: Some(args),
                    ..cmd.clone()
                };
            }
            cmds.push(cmd)
        }
        if inet6 {
            let mut cmd = KeaCommand {
                command: "lease6-get-all".to_owned(),
                service: vec!["dhcp6".to_owned()],
                ..default_cmd.clone()
            };
            // Get leases for a specified Vm.
            if let Some(ref vm) = vm {

                // DEV WARNING: 
                // Inconsistency in hostname storage between kea dhcp6 and dhcp4 :
                // ipv6 -> with a trailing dot "vm.random." 
                // ipv4 -> without trailing dot "vm.random"
                // let hostname = vm.name.clone() + ".";
                // let args: HashMap<String, String> = HashMap::from([
                //     ("hostname".to_owned(), hostname)
                // ]);
                // cmd = KeaCommand {
                //     command: cmd.command.replace("all", "by-hostname"),
                //     arguments: Some(args),
                //     ..cmd.clone()
                // };

                // Alternative:
                let duid = uuid_to_duid(&vm.uuid);
                let args: HashMap<String, String> = HashMap::from([
                    ("duid".to_owned(), duid)
                ]);
                cmd = KeaCommand {
                    command: cmd.command.replace("all", "by-duid"),
                    arguments: Some(args),
                    ..cmd.clone()
                };
            }
            cmds.push(cmd)
        }

        let mut leases: Vec<Lease> = vec![];
        for cmd in cmds {
            let response: Vec<RestResponse> = self.api.rest.post("/", Some(cmd.clone())).await?.to_value().await?;
            leases.extend(RestResponse::to_leases(response)?);
        }
        Ok(leases)
    }

}


impl LeaseMethods<'_> {
    pub fn delete(&mut self) -> LeaseDeleteMethods<'_> {
        LeaseDeleteMethods { api: self.api }
    }
}
pub struct LeaseDeleteMethods<'a> {
    api: &'a mut KeaDhcp,
}
#[bon]
impl LeaseDeleteMethods<'_> {
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn one(
        &mut self,
        lease: Lease,
    ) -> Result<(), VirshleError> {
        let args: HashMap<String, String> = HashMap::from([
            ("ip-address".to_owned(), lease.address.to_string()),
            // subnet_id is optional.
            // ("subnet-id".to_owned(), lease.subnet_id.to_string()),
        ]);
        let cmd = KeaCommand {
            command: "lease4-del".to_owned(),
            service: vec!["dhcp4".to_owned()],
            arguments: Some(args),
        };
        let response = self.api.rest.post("/", Some(cmd.clone())).await?;
        Ok(())
    }
    #[builder(
        finish_fn = exec, 
        on(String,into),
        on(Option<String>,into)
    )]
    pub async fn many(
        &mut self,
        inet6: bool, 
        inet4: bool,
        leases: Vec<Lease>,
    ) -> Result<(), VirshleError> {
        for e in leases {
            self.one().lease(e).exec().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn getter() -> Result<()> {

        Ok(())
    }

}
