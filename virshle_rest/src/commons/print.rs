use super::Status;

use bon::bon;
use owo_colors::OwoColorize;
use virshle_core::{Peer, VmTable};
// use spinoff::{spinners, Color, Spinner};

use indexmap::IndexMap;

// Error handling
use miette::Result;
use tracing::{error, info, trace, warn};
use virshle_error::VirshleError;

/// Print the result of an operation on a single vm.
#[derive(Default, Debug, Clone)]
pub struct Printer;

#[bon]
impl Printer {
    #[tracing::instrument(skip_all)]
    #[builder(
        finish_fn = print,
        on(String,into),
        on(Option<String>,into)
    )]
    pub fn res_vm(
        &self,
        tag: &str,
        peer: &str,
        content: &Result<VmTable, VirshleError>,
    ) -> Result<String, VirshleError> {
        let tag = format!("[{tag}]");
        let message;
        match content {
            Ok(vm) => {
                let tag = tag.green();
                let vm_name = format!("vm/{}", vm.name.bold().blue());
                message = format!("✅ {tag} succedded for {vm_name} on node {}", peer.green());
            }
            Err(e) => {
                let tag = tag.red();
                message = format!("⛔️ {tag} failed on node {}", peer.green());
            }
        }
        Ok(message.to_owned())
    }
    /// Print the result of an operation on a single vm.
    #[tracing::instrument(skip_all)]
    #[builder(
        finish_fn = print,
        on(String,into),
        on(Option<String>,into)
    )]
    pub fn vec_vm(
        &self,
        tag: &str,
        peer: &str,
        content: &Vec<VmTable>,
    ) -> Result<String, VirshleError> {
        let tag = format!("[{tag}]");
        let tag = tag.green();
        let indent = " ".repeat(2);
        let mut message = "".to_owned();

        let vms_name: Vec<String> = content
            .iter()
            .map(|e| {
                let vm_name = format!("{indent}vm/{}", e.name.bold().blue());
                vm_name
            })
            .collect();
        let vms_name = vms_name.join("\n");
        let succeeded_message = format!(
            "✅ {tag} succedded for vms [\n{}\n] on peer {}\n",
            vms_name,
            peer.green()
        );
        message += &succeeded_message;
        Ok(message.to_owned())
    }

    /// Print the result of an bulk operation on multiple vms.
    #[tracing::instrument(skip_all)]
    #[builder(
        finish_fn = print,
        on(String,into),
        on(Option<String>,into)
    )]
    pub fn indexmap(
        &self,
        tag: &str,
        peer: &str,
        content: &IndexMap<Status, Vec<VmTable>>,
    ) -> Result<String, VirshleError> {
        let tag = format!("[{tag}]");
        let indent = " ".repeat(2);

        let mut message = "".to_owned();
        for (k, v) in content.iter() {
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
                            "✅ {tag} succedded for vms [\n{}\n] on peer {}\n",
                            vms_name,
                            peer.green()
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
                            "⛔️ {tag} failed for vms [\n{}\n] on peer {}\n",
                            vms_name,
                            peer.green()
                        );
                        message += &failed_message;
                    }
                }
                _ => {}
            }
        }
        Ok(message.to_owned())
    }

    /// Print the result of an bulk operation on multiple vms.
    #[tracing::instrument(skip_all)]
    #[builder(
        finish_fn = print,
        on(String,into),
        on(Option<String>,into)
    )]
    pub fn by_peer_indexmap(
        &self,
        tag: &str,
        content: &IndexMap<Peer, IndexMap<Status, Vec<VmTable>>>,
    ) -> Result<String, VirshleError> {
        let mut message = "".to_owned();
        for (peer, content) in content.iter() {
            message += &self
                .indexmap()
                .peer(&peer.alias)
                .tag(tag)
                .content(content)
                .print()?;
        }
        Ok(message)
    }
}
