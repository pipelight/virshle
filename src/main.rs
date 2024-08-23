mod display;
mod getter;
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
