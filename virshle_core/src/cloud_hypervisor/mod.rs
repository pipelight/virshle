/*
* Cloud hypervisor compatibility layer
* https://levelup.gitconnected.com/learning-rust-http-via-unix-socket-fee3241b4340
*/
mod definition;
mod disk;
mod rand;
mod vm;

pub use disk::{Disk, DiskTemplate};
pub use vm::{Vm, VmState, VmTemplate};
