use owo_colors::OwoColorize;
use std::fmt;
use tabled::{settings::Style, Table, Tabled};
use uuid::Uuid;

// Error Handling
use log::trace;
use miette::{IntoDiagnostic, Result};

use super::display;
use crate::{
    error::VirshleError,
    resources::vm::{State, Vm},
};
use human_bytes::human_bytes;

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = match self {
            State::Running => "running".green().to_string(),
            State::Paused => "paused".yellow().to_string(),
            State::PmSuspended => "pm suspended".yellow().to_string(),
            State::ShutOff => "shutoff".red().to_string(),
            State::ShutDown => "shutdown".red().to_string(),
            State::Crashed => "crashed".red().to_string(),
            State::Blocked => "blocked".to_string(),
            State::NoState => "none".white().to_string(),
            State::Last => "last".white().to_string(),
        };
        write!(f, "{}", res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn display_state() -> Result<()> {
        println!("\n{}", State::Running);
        Ok(())
    }
    #[test]
    fn display_mock() -> Result<()> {
        // Get vms
        let vms = vec![
            Vm {
                id: 4,
                name: "TestOs".to_owned(),
                vcpu: 2,
                vram: 4_200_000,
                state: State::Crashed,
                uuid: Uuid::new_v4(),
            },
            Vm {
                id: 4,
                name: "NixOs".to_owned(),
                vcpu: 2,
                vram: 4_200_000,
                state: State::Running,
                uuid: Uuid::new_v4(),
            },
        ];

        println!("");
        display(vms)?;

        Ok(())
    }
    #[test]
    fn display_current() -> Result<()> {
        let vms = Vm::get_all()?;

        println!("");
        display(vms)?;

        Ok(())
    }
}
