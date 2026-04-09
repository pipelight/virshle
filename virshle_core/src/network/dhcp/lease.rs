use bon::{bon, builder};

use serde::{Deserialize, Serialize};

// Net primitives
use macaddr::MacAddr6;
use std::net::IpAddr;

// Error handling
use miette::Result;
use virshle_error::VirshleError;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Lease {
    pub address: IpAddr,
    pub hostname: String,
    pub mac: MacAddr6,
}

#[bon]
impl Lease {
    /// Extract the original vm name,
    /// from kea (unconsistent) hostnames in dhcp records
    #[builder(
        finish_fn = extract,
        on(String,into),
        on(Option<String>,into)
    )]
    pub fn vm_name(&self, suffix: Option<String>) -> Result<String, VirshleError> {
        let vm_name = match self.address {
            IpAddr::V4(_) | IpAddr::V6(_) => {
                let mut name = self.hostname.clone();
                if let Some(suffix) = suffix {
                    if let Some(val) = name.strip_suffix(&suffix) {
                        name = val.to_owned()
                    }
                }
                name = name.trim_end_matches('.').to_owned();
                name
            }
        };
        Ok(vm_name)
    }
    /// Extract the associated full domain name (ex: test.vm),
    /// from the kea dhcp records.
    #[builder(
        finish_fn = extract,
        on(String,into),
        on(Option<String>,into)
    )]
    pub fn domain_name(
        &self,
        suffix: Option<String>,
        vm_name: &str,
    ) -> Result<String, VirshleError> {
        let mut domain = vm_name.to_owned();
        if let Some(suffix) = &suffix {
            domain += ".";
            domain += suffix;
        }
        match self.address {
            IpAddr::V4(_) => {}
            IpAddr::V6(_) => {
                domain += ".";
            }
        };
        Ok(domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::IntoDiagnostic;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn extract_hostname_from_lease() -> Result<()> {
        let ipv4_lease = Lease {
            address: "172.10.0.1".parse().into_diagnostic()?,
            hostname: "default.vm".to_owned(),
            mac: "6e:47:a2:fb:06:78".parse().into_diagnostic()?,
        };
        let hostname_4 = ipv4_lease.vm_name().extract()?;

        let ipv6_lease = Lease {
            address: "2001:db8::1".parse().into_diagnostic()?,
            hostname: "default.vm.".to_owned(),
            mac: "6e:47:a2:fb:06:78".parse().into_diagnostic()?,
        };
        let hostname_6 = ipv6_lease.vm_name().extract()?;

        println!("{:#?}", hostname_6);
        assert_eq!(hostname_4, hostname_6);
        Ok(())
    }
}
