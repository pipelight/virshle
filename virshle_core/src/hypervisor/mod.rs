/*
* Cloud hypervisor compatibility layer
* https://levelup.gitconnected.com/learning-rust-http-via-unix-socket-fee3241b4340
*/

pub mod disk;
mod rand;
pub mod vm;

pub use disk::{Disk, DiskInfo, InitDisk};

pub mod vmm;

pub use vm::{InitData, UserData};
pub use vm::{Vm, VmConfigPlus, VmData, VmInfo, VmTable};
pub use vmm::{VmInfoResponse, VmState};
