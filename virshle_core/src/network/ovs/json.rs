use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, Value::Array};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

use virshle_error::{LibError, VirshleError, WrapError};

// Error handling
use super::Ovs;
use miette::{IntoDiagnostic, Result};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct OvsResponse {
    data: Vec<Vec<Value>>,
    headings: Vec<String>,
}

impl Ovs {
    /*
     * Convert ovs-vsctl json responces to sane and readable json.
     */
    pub fn to_json(response: &str) -> Result<Value, VirshleError> {
        let ovs_reponse: OvsResponse = serde_json::from_str(&response)?;

        // Iterate response elements
        let mut items: Vec<Value> = vec![];
        for item in ovs_reponse.data {
            let mut kv = Map::new();
            for (key, value) in ovs_reponse.headings.iter().zip(item) {
                kv.insert(key.to_owned(), Self::convert_bad_json_to_good_json(&value)?);
            }
            items.push(Value::Object(kv.clone()));
        }
        let value = Value::Array(items.clone());
        Ok(value)
    }

    /*
     * Internally used by `to_json` method.
     *
     * Flatten ovs-vsctl crazy json values into sane and usable values.
     */
    fn convert_bad_json_to_good_json(value: &Value) -> Result<Value, VirshleError> {
        if let Some(array) = value.as_array() {
            let mut array = array.clone();
            let data_type = array.remove(0);
            let data_type = data_type.as_str().unwrap();

            let data = array.remove(0).to_owned();

            return match data_type {
                // Identifier
                "uuid" => Ok(data.to_owned()),
                // Array or Vec
                "set" => {
                    let data = data.as_array().unwrap();

                    let mut new_value = vec![];
                    for item in data {
                        let mut item = item.as_array().unwrap().to_vec();
                        if !item.is_empty() {
                            let heading = item.remove(0);
                            let heading = heading.as_str().unwrap().to_owned();
                            let data = item.remove(0);
                            new_value.push(data.to_owned());
                        }
                    }
                    return Ok(Value::Array(new_value));
                }
                // Object
                "map" => {
                    // for item in array
                    let data = data.as_array().unwrap();

                    let mut new_value = Map::new();
                    for item in data {
                        let mut item = item.as_array().unwrap().to_vec();
                        if !item.is_empty() {
                            let heading = item.remove(0);
                            let heading = heading.as_str().unwrap().to_owned();
                            let data = item.remove(0);
                            new_value.insert(heading, data.to_owned());
                        }
                    }
                    return Ok(Value::Object(new_value));
                }
                _ => {
                    return Ok(Value::Null);
                }
            };
        } else if value.is_string() || value.is_boolean() || value.is_number() {
            return Ok(value.to_owned());
        } else {
            return Ok(Value::Null);
        }
    }
}
