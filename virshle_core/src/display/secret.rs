use super::display;
use owo_colors::OwoColorize;
use std::fmt;
use tabled::{settings::Style, Table, Tabled};
use uuid::Uuid;

// Error Handling
use log::trace;
use miette::{IntoDiagnostic, Result};

use crate::{
    error::VirshleError,
    resources::secret::{Secret, State},
};

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = match self {
            State::Ephemeral => "ephemeral".white().to_string(),
            State::NoEphemeral => "ephemeral".white().to_string(),
            State::Private => "ephemeral".green().to_string(),
            State::NoPrivate => "ephemeral".white().to_string(),
        };
        write!(f, "{}", res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn display_state() -> Result<()> {
        println!("\n{}", State::Private);
        Ok(())
    }
    #[test]
    fn display_mock() -> Result<()> {
        let items = vec![
            Secret {
                uuid: Uuid::new_v4(),
                state: State::Ephemeral,
                ..Default::default()
            },
            Secret {
                uuid: Uuid::new_v4(),
                state: State::NoPrivate,
                ..Default::default()
            },
        ];

        println!("");
        display(items)?;

        Ok(())
    }
    #[test]
    fn display_current() -> Result<()> {
        let items = Secret::get_all()?;

        println!("");
        display(items)?;

        Ok(())
    }
}
