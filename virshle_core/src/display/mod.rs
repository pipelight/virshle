mod node;
pub mod utils;
pub mod vm_template;
// mod secret;
pub mod vm;

// Reexport
pub use vm::VmTable;
pub use vm_template::VmTemplateTable;

// use crate::resources::Vm;

// Error Handling
use log::{log_enabled, Level};
use miette::Result;
use tabled::{
    settings::{disable::Remove, object::Columns, Style},
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
        res.with(Remove::column(Columns::single(0)));
        res.with(Style::rounded());
        println!("{}", res);
    }
    Ok(())
}
