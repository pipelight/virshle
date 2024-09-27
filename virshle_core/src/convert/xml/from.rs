use bat::PrettyPrinter;
use crossterm::{execute, style::Stylize, terminal::size};
use log::{info, log_enabled, Level};
use serde_json::{json, Map, Number, Value};
use std::fs;

use minidom::Element;
use quick_xml;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::num::ParseIntError;

// Error Handling
use crate::error::VirshleError;
use miette::{IntoDiagnostic, Report, Result};

pub fn from_xml(xml: &str) -> Result<Value, VirshleError> {
    // Add dummy root namespace to libvirt xml.
    let mut xml = format!("<libvirt xmlns=\'libvirt\'>{}</libvirt>", xml);
    xml = xml.trim().to_owned();

    let dom: Element = xml.parse()?;
    let mut map = Map::new();

    from_dom_to_value(&mut map, &dom)?;
    Ok(Value::Object(map))
}
pub fn from_dom_to_value(
    value: &mut Map<String, Value>,
    element: &Element,
) -> Result<(), VirshleError> {
    for e in element.children() {
        let mut map = Map::new();
        // Attr
        for (key, value) in e.attrs() {
            map.insert(format!("@{}", key), Value::String(value.to_owned()));
        }
        // Text
        let text = e.text().trim().to_owned();
        if !text.is_empty() {
            let number: Result<u64, ParseIntError> = text.parse();
            match number {
                Ok(x) => map.insert("#text".to_owned(), Value::Number(Number::from(x))),
                Err(_) => map.insert("#text".to_owned(), Value::String(text)),
            };
        }
        // Children
        from_dom_to_value(&mut map, e)?;
        value.insert(e.name().to_owned(), Value::Object(map));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    // Error Handling
    use miette::{IntoDiagnostic, Result};

    #[test]
    fn value_to_xml() -> Result<()> {
        let value = json!({
            "domain": {
                "@type": "kvm",
                "clock": {
                    "@sync": "localtime",
                },
                "memory": {
                    "@unit": "GiB",
                    "#text": 4,
                },
                "devices": [
                    {
                        "disk": {
                            "@type": "file",
                            "source": {
                                "@file": "/mnt/encrypted.qcow2",
                            },
                        },
                    },
                ],
            },
        });
        let string = r#"
            <domain type="kvm">
                <clock sync="localtime"></clock>
                <memory unit="GiB">4</memory>
                <devices>
                    <disk type="file">
                        <source file="/mnt/encrypted.qcow2"></source>
                    </disk>
                </devices>
            </domain>
        "#;
        let res = from_xml(string)?;
        assert_eq!(value, res);
        Ok(())
    }
}
