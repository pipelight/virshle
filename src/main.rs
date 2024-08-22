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

pub fn get_attributes(map: &mut Map<String, Value>) -> Result<Vec<(String, String)>> {
    let prefix = "@";
    let mut attributes: Vec<(String, String)> = vec![];

    for key in map.clone().keys() {
        if key.starts_with(prefix) {
            let attribute: (String, String) = (
                key.strip_prefix(prefix).unwrap().to_owned(),
                map.get(key).unwrap().as_str().unwrap().to_owned(),
            );
            map.shift_remove_entry(key);
            attributes.push(attribute);
        }
    }
    Ok(attributes)
}

pub fn print_open_tag(name: &str, attributes: &Vec<(String, String)>) -> Result<()> {
    let attributes = attributes
        .iter()
        // Separate attributes with a space
        .map(|(k, v)| format!(" {k}=\"{v}\""))
        .collect::<Vec<String>>()
        .join("");

    let open_tag = format!("<{name}") + &attributes + ">";
    println!("{}", open_tag);
    Ok(())
}

pub fn print_close_tag(name: &str) -> Result<()> {
    let close_tag = format!("</{name}>");
    println!("{}", close_tag);
    Ok(())
}

pub fn get_text(map: &mut Map<String, Value>) -> Result<Option<String>> {
    let prefix = "#";
    let mut text: Option<String> = None;
    for key in map.clone().keys() {
        if key.starts_with(prefix) {
            text = Some(map.get(key).unwrap().as_str().unwrap().to_owned());
        }
    }
    Ok(text)
}

// pub fn get_element(map: &mut Map<String, Value>) -> Result<(String,Value)> {
//     Ok(())
// }

pub fn read_value(value: &mut Value) -> Result<()> {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if value.is_object() {
                    let attributes = get_attributes(value.as_object_mut().unwrap())?;
                    let text = get_text(value.as_object_mut().unwrap())?;

                    print_open_tag(key, &attributes)?;

                    // if it is an object, check recursively
                    read_value(value)?;

                    print_close_tag(key)?;
                } else {
                    println!("<{}>: {}", key, value);
                }
            }
        }
        Value::String(string) => {}
        Value::Number(number) => {}
        Value::Array(array) => {}
        Value::Bool(e) => {}
        Value::Null => {}
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

        let res = from_toml(&string)?;
        println!("{:#?}", res);
        // assert!(res.is_ok());

        Ok(())
    }
    // #[test]
    fn from_toml_to_xml() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("examples/vm/base.toml");
        let path = path.to_str().unwrap();

        let string = fs::read_to_string(path).into_diagnostic()?;

        let value = from_toml(&string)?;

        let res = to_xml(&value)?;

        // assert!(res.is_ok());
        Ok(())
    }
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

        println!("");
        read_value(&mut value)?;

        // let res = from_toml(&string);
        // assert!(res.is_ok());

        Ok(())
    }
}
