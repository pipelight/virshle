use macaddr::MacAddr6;
use serde::{Deserialize, Serialize};
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

/*
 * Convest vm uuid to mac address.
 */
pub fn uuid_to_mac(uuid: &Uuid) -> MacAddr6 {
    let uuid_origin = uuid.to_string();
    // uuid into string
    let mut uuid = uuid.to_string();
    uuid = uuid.split("-").collect::<Vec<&str>>().join("");
    uuid = uuid[..12].to_owned();

    // string to mac like
    let mut mac = "".to_owned();
    for (i, c) in uuid.chars().enumerate() {
        mac.push_str(&c.to_string());
        // if (i + 1).is_multiple_of(2) && i < (uuid.len() - 1) {
        if (i + 1) % 2 == 0 && i < (uuid.len() - 1) {
            mac.push_str(":")
        }
    }

    // mac like to mac rfc
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

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_unix_name() -> Result<()> {
        let res = unix_name("vm-sasuke-uchiha--main");
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
}
