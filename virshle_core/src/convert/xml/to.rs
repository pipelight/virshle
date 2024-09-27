use bat::PrettyPrinter;
use crossterm::{execute, style::Stylize, terminal::size};
use log::{info, log_enabled, Level};
use serde_json::{json, Map, Number, Value};
use std::fs;

use minidom::Element;
use quick_xml;
use quick_xml::{events::Event, reader::Reader, writer::Writer};
use regex::Regex;
use std::num::ParseIntError;

// Error Handling
use crate::error::VirshleError;
use miette::{IntoDiagnostic, Report, Result};

pub fn to_xml(value: &Value) -> Result<String, VirshleError> {
    // Add dummy root namespace to libvirt xml.
    let mut root: Element = Element::builder("libvirt", "libvirt").build();
    from_value_to_dom(&mut root, value)?;

    let mut buf = vec![];
    root.write_to(&mut buf);

    let mut string = String::from_utf8(buf)?;
    string = pretty(&clean(&string)?)?;

    // Debug
    if log_enabled!(Level::Info) {
        let (cols, _) = size()?;
        let divider = "-".repeat((cols / 3).into());
        println!("{}", format!("{divider}xml{divider}").green());
        PrettyPrinter::new()
            .input_from_bytes(string.as_bytes())
            .language("xml")
            .print()?;
        println!("");
    }

    Ok(string)
}

pub fn from_value_to_dom(element: &mut Element, value: &Value) -> Result<(), VirshleError> {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                match v {
                    Value::Object(map) => {
                        let mut e = Element::bare(k, k);
                        from_value_to_dom(&mut e, v)?;
                        element.append_child(e);
                    }
                    Value::Number(v) => {
                        match k {
                            _ if k.starts_with("@") => {
                                element.set_attr(k.trim_start_matches("@"), v.to_string())
                            }
                            _ if k.starts_with("#text") => element.append_text(v.to_string()),
                            _ => {}
                        };
                    }
                    Value::String(v) => {
                        match k {
                            _ if k.starts_with("@") => {
                                element.set_attr(k.trim_start_matches("@"), v)
                            }
                            _ if k.starts_with("#text") => element.append_text(v),
                            _ => {
                                let mut e = Element::bare(k, k);
                                e.append_text(v);
                                element.append_child(e);
                            }
                        };
                    }
                    Value::Array(array) => {
                        let mut e = Element::bare(k, k);
                        for x in array {
                            from_value_to_dom(&mut e, x)?;
                        }
                        element.append_child(e);
                    }
                    _ => {}
                }

                let mut e = Element::bare(k, k);
                from_value_to_dom(&mut e, v)?;
            }
        }
        _ => {}
    };
    Ok(())
}
pub fn clean(string: &str) -> Result<String, VirshleError> {
    // Remove dummy root
    let re: Regex = Regex::new("<libvirt xmlns=\'libvirt\'>").unwrap();
    let mut string = re.replace_all(&string, "").to_string();
    let re: Regex = Regex::new(r"</libvirt>").unwrap();
    string = re.replace_all(&string, "").to_string();

    // Remove namespaces
    let re_names: Regex = Regex::new(r"\sxmlns='.*?'").unwrap();
    string = re_names.replace_all(&string, "").to_string();

    // Remove newline
    string = string.trim().to_owned();
    string = string.replace('\n', "").replace('\r', "");

    // Remove white spaces
    let re_spaces: Regex = Regex::new(r">\s+?<").unwrap();
    string = re_spaces.replace_all(&string, "><").to_string();

    // Remove extra spaces
    let re_spaces: Regex = Regex::new(r"\s+").unwrap();
    string = re_spaces.replace_all(&string, " ").to_string();

    Ok(string)
}
pub fn pretty(xml: &str) -> Result<String, VirshleError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    loop {
        let ev = reader.read_event();
        match ev {
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Ok(event) => writer.write_event(event),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
        .expect("Failed to parse XML");
    }
    let res = std::str::from_utf8(&*writer.into_inner())
        .expect("Failed to convert a slice of bytes to a string slice")
        .to_string();

    Ok(res)
}

#[cfg(test)]
mod test {
    use super::*;

    // Error Handling
    use miette::{IntoDiagnostic, Result};

    #[test]
    fn value_to_xml() -> Result<()> {
        let mut string = "
            <domain type=\"kvm\">
                <name>test</name>
                <clock sync=\"localtime\"/>
                <memory unit=\"GiB\">4</memory>
                <devices>
                    <disk type=\"file\">
                        <source file=\"/mnt/encrypted.qcow2\"/>
                    </disk>
                </devices>
            </domain>
        ";
        string = string.trim();

        let value = json!({
            "domain": {
                "@type": "kvm",
                "name": {
                    "#text": "test",
                },
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
        let res = to_xml(&value)?;
        println!("{}", res);
        assert_eq!(pretty(&clean(string)?)?, res);
        Ok(())
    }
}
