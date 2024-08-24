use serde_json::{json, Map, Value};
use std::fs;

// Error Handling
use crate::error::VirshleError;
use log::trace;
use miette::{IntoDiagnostic, Result};

pub fn to_xml(value: &Value) -> Result<String, VirshleError> {
    let mut string = "".to_owned();
    if let Some(map) = value.as_object() {
        for key in map.keys() {
            if let Some((k, v)) = map.get_key_value(key) {
                let mut v = v.to_owned();
                make_xml_tree(k, &mut v, &mut -1, &mut string)?;
            }
        }
    }
    Ok(string)
}

pub fn get_ressource_definitions(value: &Value) -> Result<(), VirshleError> {
    let mut base_string = "".to_owned();

    if let Some(map) = value.as_object() {
        for key in map.keys() {
            if let Some((k, v)) = map.get_key_value("domain") {
                let mut v = v.to_owned();
                make_xml_tree(k, &mut v, &mut -1, &mut base_string)?;
                println!("\n{}", base_string);
            }
            if let Some((k, v)) = map.get_key_value("network") {
                let mut v = v.to_owned();
                make_xml_tree(k, &mut v, &mut -1, &mut base_string)?;
                println!("\n{}", base_string);
            }
        }
    }

    Ok(())
}

/**
* Detect "#text" fields from a Value/Node
* to be parsed as xml text.
* Returns an list of text (value)
*/
fn get_text(map: &mut Map<String, Value>) -> Result<Option<String>, VirshleError> {
    let prefix = "#";
    let mut text: Option<String> = None;
    for key in map.clone().keys() {
        if key.starts_with(prefix) {
            let value = map.get(key).unwrap();
            text = Some(value.to_string().trim_matches('"').to_owned());
            map.shift_remove_entry(key);
        }
    }
    Ok(text)
}

/**
* Detect "@<attribute_name>" fields from a Value/Node
* to be parsed as xml attributes.
* Returns an list of attributes (name, value)
*/
fn get_attributes(
    map: &mut Map<String, Value>,
) -> Result<Option<Vec<(String, String)>>, VirshleError> {
    let prefix = "@";
    let mut attributes: Vec<(String, String)> = vec![];

    for key in map.clone().keys() {
        if key.starts_with(prefix) {
            let attribute: (String, String) = (
                key.strip_prefix(prefix).unwrap().to_owned(),
                map.get(key)
                    .unwrap()
                    .to_string()
                    // Remove extra "" due to forced string coertion
                    .trim_matches('"')
                    .to_owned(),
            );
            map.shift_remove_entry(key);
            attributes.push(attribute);
        }
    }
    if attributes.is_empty() {
        Ok(None)
    } else {
        Ok(Some(attributes))
    }
}

/**
* Print the opening tag from a Value/Node
*/
fn make_open_tag(
    name: &str,
    attributes: Option<Vec<(String, String)>>,
    text: Option<String>,
    indent_level: &mut i64,
) -> Result<String, VirshleError> {
    let ident = "  ".repeat(*indent_level as usize);
    let mut open_tag = format!("{ident}<{name}");

    if let Some(attributes) = attributes {
        let attributes = attributes
            .iter()
            // Separate attributes with a space
            .map(|(k, v)| format!(" {k}=\"{v}\""))
            .collect::<Vec<String>>()
            .join("");
        open_tag += &attributes;
    }

    open_tag += ">";

    if let Some(text) = text {
        open_tag.push_str(&text);
    }
    open_tag.push_str("\n");
    Ok(open_tag)
}

/**
* Print the closing tag from a Value/Node
*/
fn make_close_tag(name: &str, ident_level: &mut i64) -> Result<String, VirshleError> {
    let ident = "  ".repeat(*ident_level as usize);
    let close_tag = format!("{ident}</{name}>");
    Ok(close_tag)
}

/**
* Recursive function that navigates the Value and return and mutate a string to xml.
*/
pub fn make_xml_tree(
    key: &str,
    value: &mut Value,
    ident_level: &mut i64,
    base_string: &mut String,
) -> Result<(), VirshleError> {
    match value {
        Value::Object(map) => {
            let mut ident_level: i64 = *ident_level + 1;
            let text = get_text(map)?;
            let attributes = get_attributes(map)?;

            base_string.push_str(&make_open_tag(key, attributes, text, &mut ident_level)?);
            // base_string.push_str("\n");
            for (k, v) in map {
                make_xml_tree(k, v, &mut ident_level, base_string)?;
            }
            base_string.push_str(&make_close_tag(key, &mut ident_level)?);
            base_string.push_str("\n");
        }
        Value::String(value) => {
            let mut ident_level: i64 = *ident_level + 1;
            // println!("{key}{value}");
            base_string.push_str(&make_open_tag(
                key,
                None,
                Some(value.to_owned()),
                &mut ident_level,
            )?);

            base_string.push_str(&make_close_tag(key, &mut ident_level)?);
            base_string.push_str("\n");
        }
        Value::Number(value) => {
            let mut ident_level: i64 = *ident_level + 1;
            // println!("{key}{value}");
            base_string.push_str(&make_open_tag(
                key,
                None,
                Some(value.to_string()),
                &mut ident_level,
            )?);

            base_string.push_str(&make_close_tag(key, &mut ident_level)?);
            base_string.push_str("\n");
        }
        Value::Array(value) => {
            for e in value {
                make_xml_tree(key, e, ident_level, base_string)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::convert::toml::from_toml;
    use std::path::PathBuf;

    // Error Handling
    use miette::{IntoDiagnostic, Result};

    #[test]
    fn value_to_xml() -> Result<()> {
        let mut value = json!({
            "domain": {
                "@type": "kvm",
                "clock": {
                    "@sync": "localtime",
                },
                "memory": {
                    "@unit": "GiB",
                    "#text": 4,
                },
            },
        });
        let xml = to_xml(&value)?;
        println!("\n{}", xml);

        Ok(())
    }
}
