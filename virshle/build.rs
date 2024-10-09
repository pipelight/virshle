// Clap completion script generation
use clap::{CommandFactory, Parser};
use clap_complete::{generate_to, Shell};

// Filesystem manipulation
use std::env;
use std::fs;
use std::path::Path;

// Error Handling
use miette::{IntoDiagnostic, Result};

use virshle_core::cli::Cli;

/**
Generate autocompletion scripts
*/
fn main() -> Result<()> {
    // Practical outdir
    let outdir = Path::new("../autocompletion/");
    fs::create_dir_all(outdir).into_diagnostic()?;

    let mut cmd = Cli::command();
    let name = "virshle";
    // let name = cmd.get_name().to_string();
    let shells = vec![Shell::Bash, Shell::Zsh, Shell::Fish];
    for shell in shells {
        let path = generate_to(
            shell, &mut cmd, // We need to specify what generator to use
            name, outdir, // We need to specify where to write to
        )
        .into_diagnostic()?;
        println!("cargo:warning=completion file is generated: {path:?}");
    }
    Ok(())
}
