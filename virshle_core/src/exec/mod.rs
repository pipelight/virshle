use crate::cloud_hypervisor::Vm;

use owo_colors::OwoColorize;
use pipelight_exec::{Process, Status};
use std::collections::HashMap;

// Error handling
use log::{error, info, trace, warn};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub fn exec_cmds(tag: &str, cmds: Vec<String>) -> Result<(), VirshleError> {
    let tag = format!("[{tag}]");
    for cmd in cmds {
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).term().run()?;

        match res.state.status {
            Some(Status::Succeeded) => {
                let tag = tag.green();
                let message = format!("{tag}: command succeded ");
                let help = format!("{}", &res.io.stdin.unwrap().trim(),);
                trace!("{}:{}", &message, &help);
            }
            Some(Status::Failed) | Some(Status::Aborted) => {
                let tag = tag.red();
                let message = format!("{tag}: command failed ");
                let help = format!(
                    "{} -> {} ",
                    &res.io.stdin.unwrap().trim(),
                    &res.io.stderr.unwrap().trim(),
                );
                trace!("{}:{}", &message, &help);
                return Err(LibError::builder().msg(&message).help(&help).build().into());
            }
            _ => {}
        }
    }
    Ok(())
}
