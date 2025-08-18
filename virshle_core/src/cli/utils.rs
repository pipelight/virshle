use owo_colors::OwoColorize;
use spinoff::{spinners, Color, Spinner};

use std::collections::HashMap;

use crate::cli::Cli;
use crate::{Node, NodeInfo, Vm, VmInfo, VmState, VmTemplate};
use pipelight_exec::Status;

// Error handling
use miette::{IntoDiagnostic, Result};
use tracing::{error, info, trace, warn};
use virshle_error::{LibError, VirshleError, WrapError};

// Logger
use env_logger::Builder;

/// Tracing
use tracing::Level;
use tracing_subscriber::fmt::format::{Format, PrettyFields};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

/// Build tracing
pub fn set_tracer(cli: &Cli) -> Result<(), VirshleError> {
    // Set verbosity
    let verbosity: Level = cli.verbose.tracing_level().unwrap();
    let filter = format!(
        "{},{}",
        verbosity.to_string().to_lowercase(),
        "mio=error,sqlx=error,russh=error"
    );
    let subscriber = FmtSubscriber::builder()
        .with_max_level(verbosity)
        // .with_file(false)
        .with_env_filter(EnvFilter::try_new(filter).unwrap())
        .pretty()
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    Ok(())
}
/// Build logger
pub fn set_logger(cli: &Cli) -> Result<(), VirshleError> {
    // Set verbosity
    let verbosity: log::LevelFilter = cli.verbose.log_level_filter();
    // Disable sql logs
    let filter = format!(
        "{},{}",
        verbosity.to_string().to_lowercase(),
        "mio=error,sqlx=error,russh=error"
    );
    std::env::set_var("VIRSHLE_LOG", filter);
    Builder::from_env("VIRSHLE_LOG").init();

    Ok(())
}

/// Print the result of an operation on a single vm.
#[tracing::instrument]
pub fn print_response_op(
    tag: &str,
    node: &str,
    res: &Result<Vm, VirshleError>,
) -> Result<String, VirshleError> {
    let tag = format!("[{tag}]");
    let message;
    match res {
        Ok(vm) => {
            let tag = tag.green();
            let vm_name = format!("vm/{}", vm.name.bold().blue());
            message = format!("✅ {tag} succedded for {vm_name} on node {}", node.green());
        }
        Err(e) => {
            let tag = tag.red();
            message = format!("⛔️ {tag} failed on node {}", node.green());
        }
    }
    Ok(message.to_owned())
}
/// Print the result of an bulk operation on multiple vms.
#[tracing::instrument]
pub fn print_response_bulk_op(
    tag: &str,
    node: &str,
    res: &HashMap<Status, Vec<Vm>>,
) -> Result<String, VirshleError> {
    let tag = format!("[{tag}]");
    let indent = " ".repeat(2);

    let mut message = "".to_owned();
    for (k, v) in res.iter() {
        match k {
            Status::Succeeded => {
                let tag = tag.green();
                if !v.is_empty() {
                    let vms_name: Vec<String> = v
                        .iter()
                        .map(|e| {
                            let vm_name = format!("{indent}vm/{}", e.name.bold().blue());
                            vm_name
                        })
                        .collect();
                    let vms_name = vms_name.join("\n");
                    let succeeded_message = format!(
                        "✅ {tag} succedded for vms [\n{}\n] on node {}\n",
                        vms_name,
                        node.green()
                    );
                    message += &succeeded_message;
                }
            }
            Status::Failed => {
                let tag = tag.red();
                if !v.is_empty() {
                    let vms_name: Vec<String> = v
                        .iter()
                        .map(|e| {
                            let vm_name = format!("{indent}vm/{}", e.name.bold().blue());
                            vm_name
                        })
                        .collect();
                    let vms_name = vms_name.join("\n");
                    let failed_message = format!(
                        "⛔️ {tag} failed for vms [\n{}\n] on node {}\n",
                        vms_name,
                        node.green()
                    );
                    message += &failed_message;
                }
            }
            _ => {}
        }
    }
    Ok(message.to_owned())
}
