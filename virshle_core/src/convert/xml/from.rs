use bat::PrettyPrinter;
use crossterm::{execute, style::Stylize, terminal::size};
use log::{info, log_enabled, Level};
use serde_json::{json, Map, Number, Value};
use std::collections::HashMap;
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

    Ok(Value::Object(
        map.get("libvirt").unwrap().as_object().unwrap().to_owned(),
    ))
}
// if an element has multiple children puts this children into an array
pub fn from_dom_to_array(
    value: &mut Map<String, Value>,
    element: &Element,
) -> Result<(), VirshleError> {
    let mut array = vec![];
    for e in element.children() {
        let mut map = Map::new();
        from_dom_to_value(&mut map, e)?;
        array.push(Value::Object(map.to_owned()));
    }
    value.insert(element.name().to_owned(), Value::Array(array));
    Ok(())
}

pub fn from_dom_to_value(
    value: &mut Map<String, Value>,
    element: &Element,
) -> Result<(), VirshleError> {
    match element.name() {
        "caca" => {
            // Network specific
            // For arrays that or not wrapped in a tag
            // ips
            let ips: Vec<Value> = element
                .children()
                .filter(|e| e.name() == "ip")
                .map(|e| {
                    let mut map = Map::new();
                    from_dom_to_value(&mut map, e).unwrap();
                    Value::Object(map)
                })
                .collect();
            if ips.iter().next().is_some() {
                value.insert("ips".to_owned(), Value::Array(ips));
            }
        }
        "devices" => {
            from_dom_to_array(value, element);
        }
        // "ip" => {
        // Ignore
        // Is handled outside this loop
        // }
        _ => {
            let mut map = Map::new();

            // Attributes
            for (key, value) in element.attrs() {
                map.insert(format!("@{}", key), Value::String(value.to_owned()));
            }

            // If duplicate in children
            let mut dup_map: HashMap<String, Vec<Element>> = HashMap::new();
            for e in element.children() {
                if let Some(array) = dup_map.get_mut(e.name()) {
                    array.push(e.to_owned());
                } else {
                    dup_map.insert(e.name().to_owned(), vec![e.to_owned()]);
                }
            }
            for (k, v) in dup_map.iter() {
                if v.len() > 1 {}
            }

            // If the tag has children or attributes.
            if element.children().next().is_some() || element.attrs().next().is_some() {
                // Text
                let text = element.text().trim().to_owned();
                if !text.is_empty() {
                    let number: Result<u64, ParseIntError> = text.parse();
                    match number {
                        Ok(x) => map.insert("#text".to_owned(), Value::Number(Number::from(x))),
                        Err(_) => map.insert("#text".to_owned(), Value::String(text)),
                    };
                }
                // Children
                for e in element.children() {
                    from_dom_to_value(&mut map, e);
                }
                value.insert(element.name().to_owned(), Value::Object(map));

            // If the tag has no children or attributes.
            } else {
                // Text
                let text = element.text().trim().to_owned();
                if !text.is_empty() {
                    let number: Result<u64, ParseIntError> = text.parse();
                    match number {
                        Ok(x) => {
                            value.insert(element.name().to_owned(), Value::Number(Number::from(x)))
                        }
                        Err(_) => value.insert(element.name().to_owned(), Value::String(text)),
                    };
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    // Error Handling
    use miette::{IntoDiagnostic, Result};

    #[test]
    fn network_to_xml() -> Result<()> {
        let value = json!({
            "network": {
                "name": "default_6",
                "ips": [
                    {
                        "ip": {
                            "@family": "ipv4",
                        },
                    },
                    {
                        "ip": {
                            "@family": "ipv4",
                        },
                    }
                ],
            }
        });

        let string = r#"
            <network>
                <name>default_6</name>
                <ip family='ipv4'>
                </ip>
                <ip family='ipv6'>
                </ip>
            </network>
        "#;

        let res = from_xml(string)?;

        println!("{:#?}", value);
        println!("{:#?}", res);
        assert_eq!(value, res);
        Ok(())
    }

    #[test]
    fn domain_to_xml() -> Result<()> {
        let value = json!({
            "domain": {
                "name": "test",
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
                <name>test</name>
                <clock sync="localtime"></clock>
                <memory unit="GiB">4</memory>
                <devices>
                    <disk type="file">
                        <source file="/mnt/encrypted.qcow2"></source>
                    </disk>
                    <disk type="file">
                        <source file="/mnt/encrypted.qcow2"></source>
                    </disk>
                </devices>
            </domain>
        "#;

        let res = from_xml(string)?;

        // println!("{:#?}", value);
        // println!("{:#?}", res);
        assert_eq!(value, res);
        Ok(())
    }
}
