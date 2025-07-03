/*
* Cloud hypervisor compatibility layer
* https://levelup.gitconnected.com/learning-rust-http-via-unix-socket-fee3241b4340
*/

mod definition;
mod disk;
mod rand;
mod vm;
pub mod vmm_types;

pub use definition::{Definition, Template};
pub use disk::{Disk, DiskTemplate, InitDisk};
pub use vm::{
    Account, InitData, SshData, UserData, Vm, VmConfigPlus, VmData, VmInfo, VmNet, VmTemplate,
};
pub use vmm_types::{VmConfig, VmState};
