/*
* Cloud hypervisor compatibility layer
* https://levelup.gitconnected.com/learning-rust-http-via-unix-socket-fee3241b4340
*/
mod definition;
mod template;
mod vm;

pub use vm::{Vm, VmState};
