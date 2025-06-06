use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value, Value::Array};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

use virshle_error::{LibError, VirshleError, WrapError};

// Error handling
use miette::{IntoDiagnostic, Result};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct OvsResponse {
    data: Vec<Vec<Value>>,
    headings: Vec<String>,
}

/*
 * Convert ovs-vsctl json responces to sane and readable json.
 */
pub fn to_json(response: &str) -> Result<Value, VirshleError> {
    let ovs_reponse: OvsResponse = serde_json::from_str(&response)?;

    // Iterate over ovs response elements
    let mut items: Vec<Value> = vec![];
    for item in ovs_reponse.data {
        let mut kv = Map::new();
        for (key, value) in ovs_reponse.headings.iter().zip(item) {
            kv.insert(key.to_owned(), convert_bad_json_to_good_json(&value)?);
            if key == "type" {}
        }
        items.push(Value::Object(kv.clone()));
    }
    let mut value = Value::Array(items.clone());
    unflatten(&mut value)?;
    flatten(&mut value)?;

    Ok(value)
}

/*
 * Strenghten return types.
 * Force returning a Vec<String> instead of a String
 * fo arrays containing a single value (unflatten).
 */
pub fn unflatten(value: &mut Value) -> Result<(), VirshleError> {
    if let Some(array) = value.as_array_mut() {
        for e in array {
            if let Some(object) = e.as_object_mut() {
                for (key, value) in object {
                    // Cast string into vec of string
                    // See OvsBridge struct
                    if key == "ports" && value.is_string() {
                        *value = Value::Array(vec![value.to_owned()]);
                    }
                }
            }
        }
    }
    Ok(())
}
/*
 * Strenghten return types.
 * Force returning a null String instead of an empty Vec<> for null values.
 * fo arrays containing a single value (unflatten).
 */
pub fn flatten(value: &mut Value) -> Result<(), VirshleError> {
    if let Some(array) = value.as_array_mut() {
        for e in array {
            if let Some(object) = e.as_object_mut() {
                for (key, value) in object {
                    // Cast string into vec of string
                    if [
                        "mac".to_owned(),
                        "mac_in_use".to_owned(),
                        "admin_state".to_owned(),
                    ]
                    .contains(key)
                        && value.is_array()
                        && value.as_array().unwrap().to_vec().is_empty()
                    {
                        *value = Value::String("".to_owned());
                    }
                    if ["ifindex".to_owned()].contains(key)
                        && value.is_array()
                        && value.as_array().unwrap().to_vec().is_empty()
                    {
                        *value = Value::Number(Number::from_u128(0).unwrap());
                    }
                }
            }
        }
    }
    Ok(())
}

/*
 * Internally used by `to_json` method.
 *
 * Flatten ovs-vsctl crazy json values into sane and usable values.
*
 * BUG: OVS will return arrays for missing values
 * even if type is other(ex: string)
 */
fn convert_bad_json_to_good_json(value: &Value) -> Result<Value, VirshleError> {
    if value.is_array() {
        let mut array = value.as_array().unwrap().to_owned();
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
    } else if value.is_string() {
        // Safeguard: remove empty string and empty quoted strings
        let string: String = value.to_string().trim().trim_matches('"').to_owned();
        if string.is_empty() {
            return Ok(Value::Null);
        } else {
            return Ok(value.to_owned());
        }
    } else if value.is_boolean() || value.is_number() {
        return Ok(value.to_owned());
    } else {
        return Ok(Value::Null);
    }
}
