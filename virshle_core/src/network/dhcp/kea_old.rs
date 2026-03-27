// TODO: Refactor this fucking mess:
// factorize functions!!

use bon::builder;

use virshle_network::{
    connection::{Connection, TcpConnection},
    http::{Rest, RestClient},
};

use serde::{Deserialize, Serialize};
use std::str::FromStr;

// IP
use macaddr::MacAddr6;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::Vm;
use std::collections::HashMap;

// Error handling
use miette::Result;
use virshle_error::VirshleError;

use super::Lease;

impl KeaDhcp {
    pub async fn get_leases_by_hostname(&self, hostname: &str) -> Result<Vec<Lease>, VirshleError> {
        let mut leases: Vec<Lease> = self.get_ipv6_leases_by_hostname(hostname).await?;
        leases.extend(self.get_ipv4_leases_by_hostname(hostname).await?);
        Ok(leases)
    }
    pub async fn get_ipv6_leases_by_hostname(
        &self,
        vm_name: &str,
    ) -> Result<Vec<Lease>, VirshleError> {
        let raw_leases = self._get_ipv6_leases_by_hostname(vm_name).await?;
        let leases = raw_leases.iter().map(|e| Lease::from(e)).collect();
        Ok(leases)
    }
    pub async fn _get_ipv6_leases_by_hostname(
        &self,
        vm_name: &str,
    ) -> Result<Vec<RawLease>, VirshleError> {
        let conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest: RestClient = conn.into();
        rest.open().await?;

        let hostname = self.to_domain_name(IpAddr::V6(Ipv6Addr::UNSPECIFIED), vm_name)?;
        let cmd = KeaCommand {
            command: "lease6-get-by-hostname".to_owned(),
            service: vec!["dhcp6".to_owned()],
            arguments: Some(HashMap::from([("hostname".to_owned(), hostname)])),
        };

        let mut leases: Vec<RawLease> = vec![];
        let response: Vec<RestResponse> =
            rest.post("/", Some(cmd.clone())).await?.to_value().await?;
        if let Some(inside) = response.first() {
            if let Some(arguments) = &inside.arguments {
                leases = arguments.leases.clone();
            }
        }
        Ok(leases)
    }
    pub async fn get_ipv4_leases_by_hostname(
        &self,
        vm_name: &str,
    ) -> Result<Vec<Lease>, VirshleError> {
        let raw_leases = self._get_ipv4_leases_by_hostname(vm_name).await?;
        let leases = raw_leases.iter().map(|e| Lease::from(e)).collect();
        Ok(leases)
    }
    pub async fn _get_ipv4_leases_by_hostname(
        &self,
        vm_name: &str,
    ) -> Result<Vec<RawLease>, VirshleError> {
        let conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest: RestClient = conn.into();
        rest.open().await?;

        let hostname = self.to_domain_name(IpAddr::V4(Ipv4Addr::UNSPECIFIED), vm_name)?;
        let cmd = KeaCommand {
            command: "lease4-get-by-hostname".to_owned(),
            service: vec!["dhcp4".to_owned()],
            arguments: Some(HashMap::from([("hostname".to_owned(), hostname)])),
        };

        let mut leases: Vec<RawLease> = vec![];
        let response: Vec<RestResponse> =
            rest.post("/", Some(cmd.clone())).await?.to_value().await?;
        if let Some(inside) = response.first() {
            if let Some(arguments) = &inside.arguments {
                leases = arguments.leases.clone();
            }
        }
        Ok(leases)
    }
}

impl KeaDhcp {
    pub async fn get_leases(&self) -> Result<Vec<Lease>, VirshleError> {
        let mut leases: Vec<Lease> = self.get_ipv6_leases().await?;
        leases.extend(self.get_ipv4_leases().await?);
        Ok(leases)
    }

    pub async fn get_ipv4_leases(&self) -> Result<Vec<Lease>, VirshleError> {
        let raw_leases = self._get_ipv4_leases().await?;
        let leases = raw_leases.iter().map(|e| Lease::from(e)).collect();
        Ok(leases)
    }
    pub async fn _get_ipv4_leases(&self) -> Result<Vec<RawLease>, VirshleError> {
        let conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest: RestClient = conn.into();
        rest.open().await?;

        let cmd = KeaCommand {
            command: "lease4-get-all".to_owned(),
            service: vec!["dhcp4".to_owned()],
            ..Default::default()
        };

        let mut leases: Vec<RawLease> = vec![];
        let response: Vec<RestResponse> =
            rest.post("/", Some(cmd.clone())).await?.to_value().await?;
        if let Some(inside) = response.first() {
            if let Some(arguments) = &inside.arguments {
                leases = arguments.leases.clone();
            }
        }
        Ok(leases)
    }

    pub async fn get_ipv6_leases(&self) -> Result<Vec<Lease>, VirshleError> {
        let raw_leases = self._get_ipv6_leases().await?;
        let leases = raw_leases.iter().map(|e| Lease::from(e)).collect();
        Ok(leases)
    }
    pub async fn _get_ipv6_leases(&self) -> Result<Vec<RawLease>, VirshleError> {
        let conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest: RestClient = conn.into();
        rest.open().await?;

        let cmd = KeaCommand {
            command: "lease6-get-all".to_owned(),
            service: vec!["dhcp6".to_owned()],
            ..Default::default()
        };

        let mut leases: Vec<RawLease> = vec![];
        let response: Vec<RestResponse> =
            rest.post("/", Some(cmd.clone())).await?.to_value().await?;
        if let Some(inside) = response.first() {
            if let Some(arguments) = &inside.arguments {
                leases = arguments.leases.clone();
            }
        }
        Ok(leases)
    }

    pub async fn clean_leases(&self) -> Result<(), VirshleError> {
        self.clean_ipv6_leases().await?;
        self.clean_ipv4_leases().await?;
        Ok(())
    }
    /// Remove leases if associated vm doesn't exist.
    pub async fn clean_ipv4_leases(&self) -> Result<(), VirshleError> {
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
        let mut leases = self._get_ipv4_leases().await?;

        // Remove leases if no corresponding vm name
        leases = leases
            .into_iter()
            .filter(|e| !vms.contains(&self.to_vm_name(&Lease::from(e)).unwrap()))
            .map(|e| e.to_owned())
            .collect();

        self._delete_ipv4_leases(leases).await?;
        Ok(())
    }
    /// Remove leases if associated vm doesn't exist.
    pub async fn clean_ipv6_leases(&self) -> Result<(), VirshleError> {
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
        let mut leases = self._get_ipv6_leases().await?;

        // Remove leases if no corresponding vm name
        leases = leases
            .into_iter()
            .filter(|e| !vms.contains(&self.to_vm_name(&Lease::from(e)).unwrap()))
            .map(|e| e.to_owned())
            .collect();

        self._delete_ipv6_leases(leases).await?;
        Ok(())
    }

    pub async fn delete_leases(&self, vm_name: &str) -> Result<(), VirshleError> {
        self.delete_ipv4_leases_by_name(vm_name).await?;
        self.delete_ipv6_leases_by_name(vm_name).await?;
        Ok(())
    }

    // Delete a list of leases.
    pub async fn _delete_ipv6_leases(&self, leases: Vec<RawLease>) -> Result<(), VirshleError> {
        let conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest: RestClient = conn.into();
        rest.open().await?;

        let mut vec_req_map: Vec<HashMap<String, String>> = vec![];
        for lease in leases {
            let lease = match lease {
                RawLease::V6(lease) => {
                    let req_map: HashMap<String, String> = HashMap::from([
                        ("ip-address".to_owned(), lease.address.to_string()),
                        ("subnet-id".to_owned(), lease.subnet_id.to_string()),
                    ]);
                    vec_req_map.push(req_map);
                }
                RawLease::V4(v) => {}
            };
        }
        for e in vec_req_map {
            let cmd = KeaCommand {
                command: "lease6-del".to_owned(),
                service: vec!["dhcp6".to_owned()],
                arguments: Some(e),
            };
            rest.post("/", Some(cmd.clone())).await?;
        }

        Ok(())
    }

    // Delete a list of leases.
    pub async fn _delete_ipv4_leases(&self, leases: Vec<RawLease>) -> Result<(), VirshleError> {
        let conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest: RestClient = conn.into();
        rest.open().await?;

        let mut vec_req_map: Vec<HashMap<String, String>> = vec![];
        for lease in leases {
            let lease = match lease {
                RawLease::V4(lease) => {
                    let req_map: HashMap<String, String> = HashMap::from([
                        ("ip-address".to_owned(), lease.address.to_string()),
                        ("subnet-id".to_owned(), lease.subnet_id.to_string()),
                    ]);
                    vec_req_map.push(req_map);
                }
                RawLease::V6(v) => {}
            };
        }
        for e in vec_req_map {
            let cmd = KeaCommand {
                command: "lease4-del".to_owned(),
                service: vec!["dhcp4".to_owned()],
                arguments: Some(e),
            };
            rest.post("/", Some(cmd.clone())).await?;
        }
        Ok(())
    }

    // Delete a vm leases.
    pub async fn delete_ipv6_leases_by_name(&self, vm_name: &str) -> Result<(), VirshleError> {
        let conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest: RestClient = conn.into();
        rest.open().await?;

        let hostname = self.to_domain_name(IpAddr::V6(Ipv6Addr::UNSPECIFIED), vm_name)?;

        let mut vec_req_map: Vec<HashMap<String, String>> = vec![];
        let leases = self._get_ipv6_leases_by_hostname(&hostname).await?;
        for lease in leases {
            let lease = match lease {
                RawLease::V6(lease) => {
                    let req_map: HashMap<String, String> = HashMap::from([
                        ("ip-address".to_owned(), lease.address.to_string()),
                        ("subnet-id".to_owned(), lease.subnet_id.to_string()),
                    ]);
                    vec_req_map.push(req_map);
                }
                RawLease::V4(v) => {}
            };
        }

        for e in vec_req_map {
            let cmd = KeaCommand {
                command: "lease6-del".to_owned(),
                service: vec!["dhcp6".to_owned()],
                arguments: Some(e),
            };
            rest.post("/", Some(cmd.clone())).await?;
        }

        Ok(())
    }

    // Delete a vm leases.
    pub async fn delete_ipv4_leases_by_name(&self, vm_name: &str) -> Result<(), VirshleError> {
        let conn = Connection::TcpConnection(TcpConnection::new(&self.url.clone().unwrap())?);
        let mut rest: RestClient = conn.into();
        rest.open().await?;

        let hostname = self.to_domain_name(IpAddr::V4(Ipv4Addr::UNSPECIFIED), vm_name)?;

        let mut vec_req_map: Vec<HashMap<String, String>> = vec![];
        let leases = self._get_ipv4_leases_by_hostname(&hostname).await?;
        for lease in leases {
            let lease = match lease {
                RawLease::V4(lease) => {
                    let req_map: HashMap<String, String> = HashMap::from([
                        ("ip-address".to_owned(), lease.address.to_string()),
                        ("subnet-id".to_owned(), lease.subnet_id.to_string()),
                    ]);
                    vec_req_map.push(req_map);
                }
                RawLease::V6(v) => {}
            };
        }

        for e in vec_req_map {
            let cmd = KeaCommand {
                command: "lease4-del".to_owned(),
                service: vec!["dhcp4".to_owned()],
                arguments: Some(e),
            };
            rest.post("/", Some(cmd.clone())).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn read_leases() -> Result<()> {
        let res = KeaDhcp::default().get_leases().await?;
        println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn read_leases4() -> Result<()> {
        let res = KeaDhcp::default().get_ipv4_leases().await?;
        println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn read_leases6() -> Result<()> {
        let res = KeaDhcp::default().get_ipv6_leases().await?;
        println!("{:#?}", res);
        Ok(())
    }

    #[tokio::test]
    async fn read_leases6() -> Result<()> {
        let res = KeaDhcp::default().get_ipv6_leases().await?;

        println!("{:#?}", res);
        Ok(())
    }
    #[tokio::test]
    async fn extract_hostname_from_lease() -> Result<(), VirshleError> {
        let ipv4_lease = Lease {
            address: IpAddr::V4(Ipv4Addr::from_str("172.10.0.1").unwrap()),
            hostname: "default".to_owned(),
            mac: MacAddr6::from_str("6e:47:a2:fb:06:78").unwrap(),
        };
        let hostname_4 = ipv4_lease.to_vm_name()?;

        let ipv6_lease = Lease {
            address: IpAddr::V4(Ipv4Addr::from_str("172.10.0.1").unwrap()),
            hostname: "default".to_owned(),
            mac: MacAddr6::from_str("6e:47:a2:fb:06:78").unwrap(),
        };
        let hostname_6 = ipv6_lease.to_vm_name()?;
        println!("{:#?}", hostname_6);

        assert_eq!(hostname_4, hostname_6);

        Ok(())
    }
    #[tokio::test]
    async fn read_leases6() -> Result<()> {
        let res = KeaDhcp::default().get_ipv6_leases().await?;

        println!("{:#?}", res);
        Ok(())
    }
}
