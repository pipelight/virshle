use std::{process::ExitCode, u8};
use virshle_core::cli::Cli;
use virt;

// Error Handling
use log::trace;
use miette::{IntoDiagnostic, MietteHandlerOpts, Result, RgbColors};

/**
The binary entrypoint.
This main function is the first function to be executed when launching the binary.
*/
#[tokio::main]
async fn main() -> Result<()> {
    trace!("Launch process.");
    make_handler()?;
    Cli::run().await?;
    trace!("Process clean exit.");
    Ok(())
}

/**
The make handler functions is executed right after the main function
to set up a verbose and colorful error/panic handler.
*/
pub fn make_handler() -> Result<()> {
    virt::error::clear_error_callback();
    miette::set_hook(Box::new(|_| {
        Box::new(
            MietteHandlerOpts::new()
                .rgb_colors(RgbColors::Never)
                .color(true)
                .unicode(true)
                .terminal_links(true)
                .context_lines(3)
                .with_cause_chain()
                .build(),
        )
    }))?;
    miette::set_panic_hook();
    Ok(())
}
