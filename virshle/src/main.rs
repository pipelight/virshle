use cli::Cli;
use std::{process::ExitCode, u8};

// Error Handling
use log::trace;
use miette::{IntoDiagnostic, Result};

/**
The binary entrypoint.
This main function is the first function to be executed when launching the binary.
*/
fn main() -> Result<()> {
    trace!("Launch process.");
    make_handler()?;
    Cli::run()?;
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
