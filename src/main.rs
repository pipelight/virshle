use pipelight_utils::files::*;
use serde_json::{json, Map, Value};

use quick_xml::events::{attributes, BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;

// Filesystem - read file
use std::fs;
// Error Handling
use miette::{IntoDiagnostic, Result};

/**
Returns a Config struct from a provided toml file path.
*/

pub fn from_toml(string: &str) -> Result<Value> {
    let res = toml::from_str(string);
    match res {
        Ok(res) => Ok(res),
        Err(e) => {
            let err = TomlError::new(e, &string);
            Err(err.into())
        }
    }
}

pub fn to_xml(value: &Value) -> Result<String> {
    let mut w_root = Map::new();
    w_root.insert("root".to_owned(), value.to_owned());

    let value = Value::Object(w_root);
    // println!("{:#?}", value);

    let res = quick_xml::se::to_string(&value).into_diagnostic()?;
    Ok(res)
}

pub fn get_text(map: &mut Map<String, Value>) -> Result<Option<String>> {
    let prefix = "#";
    let mut text: Option<String> = None;
    for key in map.clone().keys() {
        if key.starts_with(prefix) {
            let value = map.get(key).unwrap();
            text = Some(value.to_string().trim_matches('"').to_owned());
        }
    }
    Ok(text)
}

pub fn get_attributes(map: &mut Map<String, Value>) -> Result<Option<Vec<(String, String)>>> {
    let prefix = "@";
    let mut attributes: Vec<(String, String)> = vec![];

    for key in map.clone().keys() {
        if key.starts_with(prefix) {
            let attribute: (String, String) = (
                key.strip_prefix(prefix).unwrap().to_owned(),
                map.get(key).unwrap().to_string(),
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

pub fn print_open_tag(
    name: &str,
    attributes: Option<Vec<(String, String)>>,
    text: Option<String>,
    indent_level: &mut i64,
) -> Result<()> {
    let ident = "  ".repeat(*indent_level as usize);
    let mut open_tag = format!("{ident}<{name}");

    if let Some(attributes) = attributes {
        let attributes = attributes
            .iter()
            // Separate attributes with a space
            .map(|(k, v)| format!(" {k}={v}"))
            .collect::<Vec<String>>()
            .join("");
        open_tag += &attributes;
    }

    open_tag += ">";

    if let Some(text) = text {
        open_tag += &format!("{}", text);
    }

    println!("{}", open_tag);
    Ok(())
}

pub fn print_close_tag(name: &str, ident_level: &mut i64) -> Result<()> {
    let ident = "  ".repeat(*ident_level as usize);
    let close_tag = format!("{ident}</{name}>");
    println!("{}", close_tag);
    Ok(())
}

// pub fn get_element(map: &mut Map<String, Value>) -> Result<(String,Value)> {
//     Ok(())
// }

pub fn read_value(key: &str, value: &mut Value, ident_level: &mut i64) -> Result<()> {
    match value {
        Value::Object(map) => {
            let mut ident_level: i64 = *ident_level + 1;
            let text = get_text(map)?;
            let attributes = get_attributes(map)?;

            print_open_tag(key, attributes, text, &mut ident_level)?;
            for (k, v) in map {
                read_value(k, v, &mut ident_level)?;
            }
            print_close_tag(key, &mut ident_level)?;
        }
        Value::String(value) => {
            print_open_tag(key, None, None, ident_level)?;
            let ident = "  ".repeat(*ident_level as usize);
            println!("{ident}{value}");
            print_close_tag(key, ident_level)?;
        }
        Value::Array(value) => {
            for e in value {
                read_value(key, e, ident_level)?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    // Error Handling
    use miette::{IntoDiagnostic, Result};

    #[test]
    fn read_file_to_string() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("examples/vm/base.toml");
        let path = path.to_str().unwrap();
        let string = fs::read_to_string(path).into_diagnostic()?;
        Ok(())
    }
    #[test]
    fn load_toml_file() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("examples/vm/base.toml");
        let path = path.to_str().unwrap();

        let string = fs::read_to_string(path).into_diagnostic()?;

        let res = from_toml(&string);
        // println!("{:#?}", res);
        assert!(res.is_ok());

        Ok(())
    }
    #[test]
    fn from_toml_to_xml() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("examples/vm/base.toml");
        let path = path.to_str().unwrap();

        let string = fs::read_to_string(path).into_diagnostic()?;

        let mut value = from_toml(&string)?;
        println!("{:#?}", value);

        println!("");
        // let res = to_xml(&value)?;

        read_value("root", &mut value, &mut 0)?;
        // assert!(res.is_ok());
        Ok(())
    }
    // #[test]
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

        println!("");
        read_value("root", &mut value, &mut 0)?;

        // let res = from_toml(&string);
        // assert!(res.is_ok());

        Ok(())
    }
}
