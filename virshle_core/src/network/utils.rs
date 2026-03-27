use macaddr::MacAddr6;
use std::str::FromStr;
use uuid::Uuid;

// Error handling
use miette::{IntoDiagnostic, Result};
use tracing::{debug, info, trace};
use virshle_error::{LibError, VirshleError};

/*
* Shorten an interface name to Unix MAX_LENGTH.
* Unix iface can' have names longer than 15 chars.
*/
pub fn unix_name(name: &str) -> String {
    let res = if name.len() > 15 {
        name[..15].to_owned()
    } else {
        name.to_owned()
    };
    res
}

/// Convert Vm uuid to predictable mac address.
pub fn uuid_to_mac(uuid: &Uuid) -> MacAddr6 {
    let uuid_origin = uuid.to_string();
    // uuid into string
    let mut uuid = uuid.to_string();
    uuid = uuid.split("-").collect::<Vec<&str>>().join("");
    uuid = uuid[..12].to_owned();

    // hexadecimal string to MAC like string
    let mut mac = "".to_owned();
    for (i, c) in uuid.chars().enumerate() {
        mac.push_str(&c.to_string());
        // if (i + 1).is_multiple_of(2) && i < (uuid.len() - 1) {
        if (i + 1) % 2 == 0 && i < (uuid.len() - 1) {
            mac.push_str(":")
        }
    }

    // Convert:
    // - from MAC like string
    // - to rfc complient hardware address.
    let mut chars = mac.chars().collect::<Vec<char>>();
    chars[1] = 'e';
    mac = chars.iter().collect();

    let mac = MacAddr6::from_str(&mac).unwrap();

    trace!(
        "converted uuid: {:#?} to mac: {:#?}",
        uuid_origin,
        mac.to_string()
    );
    mac
}

/// Convert Vm uuid to predictable dhcp duid-uuid.
pub fn uuid_to_duid(uuid: &Uuid) -> String {
    let uuid_origin = uuid.to_string();

    // memo (16 bits - 2 bytes - 4 hex chars)
    let duid_type = format!("{:04x}", 4); // must yield "0004"

    // uuid into string
    let mut uuid = uuid.to_string();
    uuid = uuid.split("-").collect::<Vec<&str>>().join("");

    let raw_duid = duid_type + &uuid;
    // No need to slice the uuid because it already has the required length.
    // memo (128 bits - 16 bytes - 32 hex chars)

    // hexadecimal string to MAC like string
    let mut duid = "".to_owned();
    for (i, c) in raw_duid.chars().enumerate() {
        duid.push_str(&c.to_string().to_uppercase());
        // if (i + 1).is_multiple_of(2) && i < (uuid.len() - 1) {
        if (i + 1) % 2 == 0 && i < (raw_duid.len() - 1) {
            duid.push_str(":")
        }
    }
    duid
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_unix_name() -> Result<()> {
        let res = unix_name("vm-sasuke_uchiha--main");
        assert_eq!(&res, "vm-sasuke_uchih");
        Ok(())
    }

    #[test]
    fn test_uuid_to_mac() -> Result<()> {
        let uuid = Uuid::parse_str("c37b3266-9c59-42bb-8ecf-bdd643236a78").unwrap();
        let mac = uuid_to_mac(&uuid);
        assert_eq!(mac.to_string(), "CE:7B:32:66:9C:59");
        println!("{:#?}", mac.to_string());
        Ok(())
    }
    #[test]
    fn test_uuid_to_duid() -> Result<()> {
        let uuid = Uuid::parse_str("c37b3266-9c59-42bb-8ecf-bdd643236a78").unwrap();
        let duid = uuid_to_duid(&uuid);
        assert_eq!(
            duid,
            "00:04:C3:7B:32:66:9C:59:42:BB:8E:CF:BD:D6:43:23:6A:78"
        );
        Ok(())
    }
}
