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
    resources::net::{Net, State},
};

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = match self {
            State::Active => "active".green().to_string(),
            State::Inactive => "inactive".red().to_string(),
        };
        write!(f, "{}", res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn display_state() -> Result<()> {
        println!("\n{}", State::Active);
        Ok(())
    }
    #[test]
    fn display_mock() -> Result<()> {
        let items = vec![
            Net {
                uuid: Uuid::new_v4(),
                name: "net_arch".to_owned(),
                state: State::Active,
                ..Default::default()
            },
            Net {
                uuid: Uuid::new_v4(),
                name: "net_nix".to_owned(),
                state: State::Inactive,
                ..Default::default()
            },
        ];

        println!("");
        display(items)?;

        Ok(())
    }
    #[test]
    fn display_current() -> Result<()> {
        let items = Net::get_all()?;

        println!("");
        display(items)?;

        Ok(())
    }
}
