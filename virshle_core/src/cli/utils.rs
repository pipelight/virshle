use owo_colors::OwoColorize;
use spinoff::{spinners, Color, Spinner};

use std::collections::HashMap;

use crate::{Node, NodeInfo, Vm, VmInfo, VmState, VmTemplate};
use pipelight_exec::Status;

// Error handling
use log::{error, info, trace, warn};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

/// Print the result of an operation on a single vm.
pub fn print_response_op(
    sp: &mut Spinner,
    tag: &str,
    node: &str,
    res: &Result<Vm, VirshleError>,
) -> Result<(), VirshleError> {
    let tag = format!("[{tag}]");
    match res {
        Ok(vm) => {
            let tag = tag.green();
            let vm_name = format!("vm/{}", vm.name.bold().blue());
            let message = format!("✅ {tag} succedded for {vm_name} on node {}", node.green());
            sp.stop_and_persist("✅", &message);
        }
        Err(e) => {
            let tag = tag.red();
            let message = format!("{tag} failed on node {}", node.green());
            sp.stop_and_persist("⛔️", &message);
            println!("{}", e);
        }
    }
    Ok(())
}
/// Print the result of an bulk operation on multiple vms.
pub fn print_response_bulk_op(
    sp: &mut Spinner,
    tag: &str,
    node: &str,
    res: &HashMap<Status, Vec<Vm>>,
) -> Result<(), VirshleError> {
    let tag = format!("[{tag}]");
    let indent = " ".repeat(2);

    let mut messages = "".to_owned();
    for (k, v) in res.iter() {
        match k {
            Status::Succeeded => {
                let tag = tag.green();
                if !v.is_empty() {
                    let vms_name: Vec<String> = v
                        .iter()
                        .map(|e| {
                            let vm_name = format!("{indent}vm/{}", e.name.bold().blue());
                            vm_name
                        })
                        .collect();
                    let vms_name = vms_name.join("\n");
                    let message = format!(
                        "✅ {tag} succedded for vms [\n{}\n] on node {}\n",
                        vms_name,
                        node.green()
                    );
                    messages += &message;
                }
            }
            Status::Failed => {
                let tag = tag.red();
                if !v.is_empty() {
                    let vms_name: Vec<String> = v
                        .iter()
                        .map(|e| {
                            let vm_name = format!("{indent}vm/{}", e.name.bold().blue());
                            vm_name
                        })
                        .collect();
                    let vms_name = vms_name.join("\n");
                    let message = format!(
                        "⛔️ {tag} failed for vms [\n{}\n] on node {}\n",
                        vms_name,
                        node.green()
                    );
                    messages += &message;
                }
            }
            _ => {}
        }
    }
    sp.stop_and_persist(&messages, "");
    Ok(())
}
