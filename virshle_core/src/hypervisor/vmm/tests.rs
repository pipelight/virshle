use super::*;

use crate::utils::testing;
use crate::{config::DiskTemplate, VmTemplate};
use pretty_assertions::assert_eq;

/// Create a testing Vm to try Vmm methods.
fn testing_vm() -> Result<Vm, VirshleError> {
    let template = VmTemplate {
        name: "test".to_owned(),
        vcpu: 1,
        vram: "1GiB".to_owned(),
        uuid: None,
        disk: Some(vec![DiskTemplate {
            name: "os".to_owned(),
            path: "/var/lib/virshle/cache/nixos.xxs.efi.img".to_owned(),

            readonly: None,
        }]),
        net: None,
        extra: None,
    };
    let vm: Vm = template.try_into()?;
    Ok(vm)
}

#[tokio::test]
async fn test_vmm() -> Result<()> {
    testing::tracer()
        .verbosity(tracing::Level::TRACE)
        .db(true)
        .set()?;

    // Create and start a testing Vm
    let mut vm = testing_vm()?;
    vm.create(None).await?;
    vm.start(None, None).await?;

    // Ping
    let res: Result<(), VirshleError> = vm.vmm().api()?.ping().await;
    assert!(res.is_ok());
    testing::unwind(res)?;

    // Info
    let res: Result<VmInfoResponse, VirshleError> = vm.vmm().api()?.info().await;
    assert!(res.is_ok());
    testing::unwind(res)?;
    // State
    let res: Result<VmState, VirshleError> = vm.vmm().api()?.state().await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), VmState::Running);
    // Pause
    let res: Result<(), VirshleError> = vm.vmm().api()?.pause().await;
    assert!(res.is_ok());
    let res: Result<VmState, VirshleError> = vm.vmm().api()?.state().await;
    assert_eq!(res.unwrap(), VmState::Paused);
    // Shutdown
    let res: Result<(), VirshleError> = vm.vmm().api()?.shutdown().await;
    assert!(res.is_ok());
    let res: Result<VmState, VirshleError> = vm.vmm().api()?.state().await;
    assert_eq!(res.unwrap(), VmState::Created);
    // Boot
    let res: Result<(), VirshleError> = vm.vmm().api()?.boot().await;
    assert!(res.is_ok());
    testing::unwind(res)?;

    // Delete
    let res: Result<(), VirshleError> = vm.vmm().api()?.delete().await;
    assert!(res.is_ok());
    let res: Result<VmState, VirshleError> = vm.vmm().api()?.state().await;
    assert_eq!(res.unwrap(), VmState::NotCreated);

    // Get a valide State (process NotCreated) even when Vmm process is not Running
    // and  therefore cannot respond.
    vm.vmm().kill_process()?;
    let res: Result<VmState, VirshleError> = vm.vmm().api()?.state().await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), VmState::NotCreated);

    // Delete the testing Vm
    vm.delete().await?;
    Ok(())
}
