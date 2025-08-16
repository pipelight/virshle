use owo_colors::OwoColorize;
use pipelight_exec::{Process, Status};

// Error handling
use log::{error, info, trace, warn};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub fn exec_cmds(tag: &str, cmds: Vec<String>) -> Result<(), VirshleError> {
    for cmd in cmds {
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).term().run()?;

        match res.state.status {
            Some(Status::Failed) => {
                let tag = format!("[{tag}]");
                let message = format!("{}: command failed ", tag.red());
                let help = format!(
                    "{} -> {} ",
                    &res.io.stdin.unwrap().trim(),
                    &res.io.stderr.unwrap().trim(),
                );
                warn!("{}:{}", &message, &help);
                return Err(LibError::builder().msg(&message).help(&help).build().into());
            }
            _ => {
                let tag = format!("[{tag}]");
                let message = format!("{}: command succeded ", tag.green());
                let help = format!("{}", &res.io.stdin.unwrap().trim(),);
                trace!("{}:{}", &message, &help);
            }
        }
    }
    Ok(())
}
