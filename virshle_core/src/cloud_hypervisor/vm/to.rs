// Struct
use super::vmm_types::{CpusConfig, DiskConfig, MemoryConfig, NetConfig, VhostMode, VmConfig};
use super::{Disk, Vm, VmNet};
use std::path::PathBuf;

// Cloud Hypervisor
use uuid::Uuid;

// Error Handling
use log::info;
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError};

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    // #[test]
    fn make_vm_from_template() -> Result<()> {
        let toml = r#"
            name = "test_xs_1"
            vcpu = 1
            vram = 2

            [config]
            autostart = true
        "#;

        let item = Vm::from_toml(&toml)?.to_vmm_config()?;
        println!("{:#?}", item);
        Ok(())
    }
    #[test]
    fn make_vm_from_definition_with_ids() -> Result<()> {
        let toml = r#"

            name = "test_xs"
            uuid = "b30458d1-7c7f-4d06-acc2-159e43892e87"

            vcpu = 1
            vram = 2

            [[net]]
            [net.tap]
            name = "macvtap0"

            "#;
        let item = Vm::from_toml(&toml)?.to_vmm_config()?;
        println!("{:#?}", item);
        Ok(())
    }
}
