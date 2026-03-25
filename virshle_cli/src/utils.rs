use owo_colors::OwoColorize;
// use spinoff::{spinners, Color, Spinner};

use std::collections::HashMap;

use crate::{Node, NodeInfo, Vm, VmState, VmTable, VmTemplate};
use pipelight_exec::Status;

// Error handling
use miette::Result;
use tracing::{error, info, trace, warn};
use virshle_error::VirshleError;

/// Print the result of an operation on a single vm.
#[tracing::instrument(skip(res))]
pub fn print_response_op(
    tag: &str,
    node: &str,
    res: &Result<VmTable, VirshleError>,
) -> Result<String, VirshleError> {
    let tag = format!("[{tag}]");
    let message;
    match res {
        Ok(vm) => {
            let tag = tag.green();
            let vm_name = format!("vm/{}", vm.name.bold().blue());
            message = format!("✅ {tag} succedded for {vm_name} on node {}", node.green());
        }
        Err(e) => {
            let tag = tag.red();
            message = format!("⛔️ {tag} failed on node {}", node.green());
        }
    }
    Ok(message.to_owned())
}
/// Print the result of an bulk operation on multiple vms.
#[tracing::instrument(skip(res))]
pub fn print_response_bulk_op(
    tag: &str,
    node: &str,
    res: &HashMap<Status, Vec<Vm>>,
) -> Result<String, VirshleError> {
    let tag = format!("[{tag}]");
    let indent = " ".repeat(2);

    let mut message = "".to_owned();
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
                    let succeeded_message = format!(
                        "✅ {tag} succedded for vms [\n{}\n] on node {}\n",
                        vms_name,
                        node.green()
                    );
                    message += &succeeded_message;
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
                    let failed_message = format!(
                        "⛔️ {tag} failed for vms [\n{}\n] on node {}\n",
                        vms_name,
                        node.green()
                    );
                    message += &failed_message;
                }
            }
            _ => {}
        }
    }
    Ok(message.to_owned())
}
