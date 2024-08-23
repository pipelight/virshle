use std::path::PathBuf;

use clap::{Parser, Subcommand};

use clap_verbosity_flag::{InfoLevel, Verbosity};

/// Le CLI
#[derive(Debug, Parser)]
struct Cli {}
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    create: Option<String>,

    /// Optional name to operate on
    delete: Option<String>,

    /// List ressources
    list: Option<String>,

    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}
