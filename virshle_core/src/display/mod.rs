mod net;
// mod secret;
pub mod vm;

// use crate::resources::Vm;

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
