use owo_colors::OwoColorize;
use spinoff::{spinners, Color, Spinner};

use std::collections::HashMap;

use crate::cli::Cli;
use crate::{Node, NodeInfo, Vm, VmInfo, VmState, VmTable, VmTemplate};
use pipelight_exec::Status;

// Error handling
use miette::{IntoDiagnostic, Result};
use tracing::{error, info, trace, warn};
use virshle_error::{LibError, VirshleError, WrapError};

// Logger
use env_logger::Builder;

/// Tracing
use tracing_subscriber::fmt::format::{Format, PrettyFields};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub fn tracing_per_crate(verbosity: tracing::Level) -> Result<String, VirshleError> {
    let res = match verbosity {
        tracing::Level::TRACE => "",
        tracing::Level::DEBUG => {
            "mio=error,sqlx=error,sea_orm=info,tower_http=info,russh=error,users=warn"
        }
        tracing::Level::INFO => {
            "mio=error,sqlx=error,sea_orm=info,tower_http=info,russh=error,users=warn"
        }
        tracing::Level::WARN => {
            "mio=error,sqlx=error,sea_orm=warn,tower_http=warn,russh=error,users=warn"
        }
        tracing::Level::ERROR => {
            "mio=error,sqlx=error,sea_orm=error,tower_http=error,russh=error,users=warn"
        }
        _ => "mio=off,sqlx=off,sea_orm=off,tower_http=off,russh=off,users=off",
    };
    Ok(res.to_owned())
}
pub fn logging_per_crate(verbosity: log::LevelFilter) -> Result<String, VirshleError> {
    let res = match verbosity {
        log::LevelFilter::Trace => "",
        log::LevelFilter::Debug => {
            "mio=error,sqlx=error,sea_orm=info,tower_http=info,russh=error,users=warn"
        }
        log::LevelFilter::Info => {
            "mio=error,sqlx=error,sea_orm=info,tower_http=info,russh=error,users=warn"
        }
        log::LevelFilter::Warn => {
            "mio=error,sqlx=error,sea_orm=warn,tower_http=warn,russh=error,users=warn"
        }
        log::LevelFilter::Error => {
            "mio=error,sqlx=error,sea_orm=error,tower_http=error,russh=error,users=warn"
        }
        _ => "mio=off,sqlx=off,sea_orm=off,tower_http=off,russh=off,users=off",
    };
    Ok(res.to_owned())
}
/// Build tracing
pub fn set_tracer(cli: &Cli) -> Result<(), VirshleError> {
    // Set verbosity
    let verbosity: tracing::Level = cli.verbose.tracing_level().unwrap();
    let filter = format!(
        "{},{}",
        verbosity.to_string().to_lowercase(),
        tracing_per_crate(verbosity)?
    );
    let builder = FmtSubscriber::builder()
        .with_max_level(verbosity)
        // .with_file(false)
        .with_env_filter(EnvFilter::try_new(filter).unwrap());

    #[cfg(debug_assertions)]
    let builder = builder.pretty();
    #[cfg(not(debug_assertions))]
    let builder = builder.compact();

    let subscriber = builder.finish();
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
        logging_per_crate(verbosity)?
    );
    std::env::set_var("VIRSHLE_LOG", filter);
    Builder::from_env("VIRSHLE_LOG").init();

    Ok(())
}

/// Print the result of an operation on a single vm.
#[tracing::instrument(skip(res))]
pub fn print_response_op(
    tag: &str,
    node: &str,
    res: &Result<VmTable, VirshleError>,
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
#[tracing::instrument(skip(res))]
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
