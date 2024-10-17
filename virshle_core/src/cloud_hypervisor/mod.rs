/*
* Cloud hypervisor compatibility layer
* https://levelup.gitconnected.com/learning-rust-http-via-unix-socket-fee3241b4340
*/
mod definition;
mod disk;
mod net;
mod rand;
mod vm;

pub use definition::Definition;
pub use disk::{Disk, DiskTemplate};
pub use net::{Net, NetTemplate};
pub use vm::{Vm, VmState, VmTemplate};
