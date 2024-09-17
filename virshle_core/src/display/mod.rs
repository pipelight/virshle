// Error Handling
use miette::Result;
use tabled::{settings::Style, Table, Tabled};

mod net;
mod secret;
mod vm;

/**
* Format vec of T to table
*/
pub fn display<T>(vec: Vec<T>) -> Result<()>
where
    T: Tabled,
{
    let mut res = Table::new(&vec);
    res.with(Style::rounded());
    println!("{}", res);
    Ok(())
}
