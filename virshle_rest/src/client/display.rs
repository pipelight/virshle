use owo_colors::OwoColorize;

pub use pipelight_exec::Status;
use virshle_core::{
    config::{VmTemplate, VmTemplateTable},
    hypervisor::{
        vm::{UserData, Vm, VmInfo, VmTable},
        vmm::types::{VmInfoResponse, VmState},
    },
    peer::{NodeInfo, Peer},
};

// Error handling
use miette::Result;
use tokio::task::JoinError;
use tracing::{info, warn};
use virshle_error::VirshleError;

#[tracing::instrument]
pub async fn op_result(vm: Vm, result: Result<VmTable, VirshleError>) -> Result<(Status, VmTable)> {
    match result {
        Err(_) => {
            let vm = VmTable::from(&vm).await?;
            Ok((Status::Failed, vm))
        }
        Ok(vm) => Ok((Status::Succeeded, vm)),
    }
}

#[tracing::instrument]
pub async fn op_results(
    vm: Vm,
    result: Result<VmTable, VirshleError>,
) -> Result<(Status, VmTable)> {
    match result {
        Err(_) => {
            let vm = VmTable::from(&vm).await?;
            Ok((Status::Failed, vm))
        }
        Ok(vm) => Ok((Status::Succeeded, vm)),
    }
}
