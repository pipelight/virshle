mod net;
mod secret;
mod vm;

use crate::resources::Vm;

// Error Handling
use log::{log_enabled, Level};
use miette::Result;
use tabled::{
    settings::{object::Columns, Disable, Style},
    Table, Tabled,
};

/**
* Format vec of T to table
*/
pub fn default<T>(vec: Vec<T>) -> Result<()>
where
    T: Tabled,
{
    if log_enabled!(Level::Warn) {
        let mut res = Table::new(&vec);
        res.with(Style::rounded());
        println!("{}", res);
    } else {
        let mut res = Table::new(&vec);
        res.with(Disable::column(Columns::single(0)));
        res.with(Style::rounded());
        println!("{}", res);
    }
    Ok(())
}

pub fn vm(vec: Vec<Vm>) -> Result<()> {
    if log_enabled!(Level::Info) {
        let mut res = Table::new(&vec);
        res.with(Style::rounded());
        println!("{}", res);
    } else if log_enabled!(Level::Warn) {
        let mut res = Table::new(&vec);
        res.with(Style::rounded());
        res.with(Disable::column(Columns::last()));
        println!("{}", res);
    } else {
        let mut res = Table::new(&vec);
        res.with(Disable::column(Columns::last()));
        res.with(Disable::column(Columns::last()));
        res.with(Style::rounded());
        println!("{}", res);
    }
    Ok(())
}
