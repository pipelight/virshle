use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{fs, u32};
use tabled::Tabled;
use uuid::Uuid;

// Error Handling
use crate::error::{VirshleError, VirtError, WrapError};
use log::trace;
use miette::{IntoDiagnostic, Result};
use std::collections::HashMap;

// libvirt
use super::connect;
use crate::convert;
use convert_case::{Case, Casing};
use strum::{EnumIter, IntoEnumIterator};
use virt::secret::Secret as VirtSecret;

use once_cell::sync::Lazy;

static NVirConnectListAllSecretsFlags: u32 = 3;

#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq, EnumIter)]
pub enum State {
    #[default]
    Ephemeral = 0,
    NoEphemeral = 1,
    Private = 2,
    NoPrivate = 3,
}
impl From<u32> for State {
    fn from(value: u32) -> Self {
        match value {
            0 => State::Ephemeral,
            1 => State::NoEphemeral,
            2 => State::Private,
            3 => State::NoPrivate,
            _ => State::Ephemeral,
        }
    }
}

fn display_option(value: &Option<String>) -> String {
    match value {
        Some(s) => format!("{}", s),
        None => format!(""),
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq, Tabled)]
pub struct Secret {
    pub uuid: Uuid,
    pub state: State,
    #[tabled(display_with = "display_option")]
    pub value: Option<String>,
}
impl Secret {
    fn from(e: &VirtSecret) -> Result<Self, VirshleError> {
        let flag = e.get_usage_type()? as u32;
        let value: Option<String>;
        match e.get_value(flag) {
            Ok(v) => {
                value = String::from_utf8(e.get_value(flag)?).ok();
            }
            Err(_) => {
                value = None;
            }
        }
        let res = Self {
            uuid: e.get_uuid()?,
            state: State::from(flag),
            value,
        };
        Ok(res)
    }
    pub fn get(uuid: &str) -> Result<Self, VirshleError> {
        let uuid = Uuid::parse_str(uuid)?;

        let conn = connect()?;
        let res = VirtSecret::lookup_by_uuid(&conn, uuid.to_owned());
        match res {
            Ok(e) => {
                let item = Self::from(&e)?;
                Ok(item)
            }
            Err(e) => Err(VirtError::new(
                &format!("No network with name {:?}", uuid),
                "Maybe you made a typo",
                e,
            )
            .into()),
        }
    }
    pub fn get_all() -> Result<Vec<Self>, VirshleError> {
        let conn = connect()?;
        let mut map: HashMap<String, Self> = HashMap::new();

        for flag in State::iter() {
            let items = conn.list_all_secrets(flag as u32)?;
            for e in items.clone() {
                let secret = Self::from(&e)?;
                let uuid = secret.clone().uuid.to_string();
                if !map.contains_key(&uuid) {
                    map.insert(uuid, secret);
                }
            }
        }
        let list: Vec<Secret> = map.into_values().collect();
        Ok(list)
    }
    pub fn set(path: &str) -> Result<(), VirshleError> {
        let toml = fs::read_to_string(path)?;

        let mut value = convert::from_toml(&toml)?;
        Self::set_multi_xml_w_value(&mut value)?;

        Ok(())
    }
    /**
     *
     * Remove secret value from definition.
     * Wether it is an array of secret or a single secret.
     *
     * So both declarations can be used in a toml file:
     *
     * ```toml
     * [[secret]]
     * value="test"
     * ```
     * or
     * ```toml
     * [secret]
     * value="test"
     * ```
     */
    pub fn set_multi_xml_w_value(json: &Value) -> Result<(), VirshleError> {
        // For array of secrets
        if json["secret"].is_array() {
            for secret in json["secret"].as_array().unwrap() {
                let mut new_map = Map::new();
                new_map.insert("secret".to_owned(), secret.to_owned());
                Self::set_xml_w_value(&Value::Object(new_map))?;
            }
        // For single secret
        } else {
            Self::set_xml_w_value(json)?;
        }

        Ok(())
    }

    pub fn set_xml_w_value(json: &Value) -> Result<(), VirshleError> {
        let conn = connect()?;
        if json.is_object() {
            let mut binding = json.to_owned();
            let mut objmut = json.to_owned();
            let objmut = objmut.as_object_mut().unwrap();
            let obj = binding.as_object().unwrap();
            if let Some(obj) = obj.get("secret") {
                if let Some(secret_value) = obj.get("value") {
                    if let Some(secret_value) = secret_value.get("#text") {
                        // println!("{:#?}", secret_value);
                        let secret_value = secret_value.as_str().unwrap();

                        // Get secret uuid
                        let uuid = obj
                            .get("uuid")
                            .unwrap()
                            .as_object()
                            .unwrap()
                            .get("#text")
                            .unwrap()
                            .as_str()
                            .unwrap();
                        let uuid = Uuid::parse_str(uuid)?;

                        // Remove entry from schema for Libvirt-XML validation
                        // and Create secret
                        objmut
                            .get_mut("secret")
                            .unwrap()
                            .as_object_mut()
                            .unwrap()
                            .shift_remove_entry("value")
                            .unwrap();

                        // Create secret
                        let xml = convert::to_xml(&Value::Object(objmut.to_owned()))?;
                        Self::set_xml(&xml)?;

                        // Set secret value by uuid
                        let bytes = secret_value.to_owned().into_bytes();
                        let virtsecret = VirtSecret::lookup_by_uuid(&conn, uuid)?;

                        // Flag 0 value not used by libvirt, but its there...
                        // https://libvirt.org/html/libvirt-libvirt-secret.html#virSecretSetValue
                        virtsecret.set_value(&bytes, 0)?;
                    }
                }
            } else {
                // Create secret
                let xml = convert::to_xml(json)?;
                Self::set_xml(&xml)?;
            }
        }
        Ok(())
    }

    pub fn set_xml(xml: &str) -> Result<(), VirshleError> {
        let conn = connect()?;
        let res = VirtSecret::define_xml(&conn, &xml, 1);
        match res {
            Ok(res) => Ok(()),
            Err(e) => Err(VirtError::new(
                "The network could not be created",
                "Try deleting the network first",
                e,
            )
            .into()),
        }
    }

    pub fn delete(uuid: &str) -> Result<(), VirshleError> {
        // Guard
        Self::get(&uuid)?;

        let conn = connect()?;

        let uuid = Uuid::parse_str(uuid)?;
        let item = VirtSecret::lookup_by_uuid(&conn, uuid.to_owned())?;

        let res = item.undefine();
        match res {
            Ok(res) => Ok(()),
            Err(e) => Err(VirtError::new("The network could not be destroyed", "", e).into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::Secret;
    use std::path::PathBuf;
    use uuid::Uuid;
    // Error Handling
    use miette::{IntoDiagnostic, Result};

    #[test]
    fn fetch_secrets() -> Result<()> {
        let items = Secret::get_all();
        println!("{:#?}", items);
        Ok(())
    }

    #[test]
    fn create_secret() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../templates/secret/base.toml");
        let path = path.display().to_string();

        Secret::set(&path)?;

        Ok(())
    }

    #[test]
    fn delete_secret() -> Result<()> {
        let uuid = "d6e7d1f8-1cca-4fb3-b985-7ca74cf7cbb9";
        Secret::delete(&uuid)?;

        Ok(())
    }
}
