mod display;
mod toml;

use crate::toml::from_toml;

use pipelight_utils::files::*;
use serde_json::{json, Map, Value};

use quick_xml::events::{attributes, BytesEnd, BytesStart, Event};
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use std::io::Cursor;

use std::{process::ExitCode, u8};

// Error Handling
use log::trace;
use miette::{IntoDiagnostic, Result};
use std::fs;

/**
The pipelight binary entrypoint.
This main function is the first function to be executed when launching pipelight.
*/
fn main() -> Result<()> {
    trace!("Launch process.");
    make_handler()?;
    trace!("Process clean exit.");
    Ok(())
}

/**
The make handler functions is executed right after the main function
to set up a verbose and colorful error/panic handler.
*/
pub fn make_handler() -> Result<()> {
    miette::set_panic_hook();
    Ok(())
}

pub fn to_xml(value: &Value) -> Result<String> {
    let mut w_root = Map::new();
    w_root.insert("root".to_owned(), value.to_owned());

    let value = Value::Object(w_root);
    // println!("{:#?}", value);

    let res = quick_xml::se::to_string(&value).into_diagnostic()?;
    Ok(res)
}
