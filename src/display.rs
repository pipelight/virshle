use bevy_reflect::{FieldIter, Reflect, Struct};
// Error Handling
use log::trace;
use miette::{IntoDiagnostic, Result};

use crate::{error::VirshleError, resources::Vm};
use vec_to_array::vec_to_array;

use radicle_term::{Color, Constraint, Element, Table, TableOptions};
use std::fmt;
use std::fmt::Display;

impl Vm {
    pub fn display(vms: Vec<Vm>) -> Result<String> {
        let mut t = Table::new(TableOptions {
            border: Some(Color::Unset),
            spacing: 3,
            ..TableOptions::default()
        });
        t.push([
            "id".to_owned(),
            "name".to_owned(),
            "vcpu".to_owned(),
            "vram".to_owned(),
        ]);
        t.divider();
        for vm in vms {
            let values: [String; 4] = [
                vm.id.to_string(),
                vm.name,
                vm.vcpu.to_string(),
                vm.vram.to_string(),
            ];
            t.push(values);
        }
        let res = t.display(Constraint::UNBOUNDED);
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_display_template() -> Result<()> {
        let vms = vec![Vm {
            id: 4,
            name: "TestOs".to_owned(),
            vcpu: 2,
            vram: 420000,
        }];
        let res = Vm::display(vms)?;
        println!("\n{}", res);
        Ok(())
    }
    #[test]
    fn try_display_real() -> Result<()> {
        let vms = Vm::get_all()?;
        let res = Vm::display(vms)?;
        println!("\n{}", res);
        Ok(())
    }
}
