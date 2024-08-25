use std::error::Error;

use miette::{Diagnostic, Report};
use pipelight_error::CastError;

use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum VirshleError {
    #[error(transparent)]
    #[diagnostic(code(virshle::io::error))]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    VirtError(#[from] VirtError),

    #[error(transparent)]
    #[diagnostic(code(virshle::libvirt::error))]
    LibVirtError(#[from] virt::error::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    WrapError(#[from] WrapError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    LibError(#[from] LibError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    CastError(#[from] CastError),
}

/**
A config error with help higher origin
Can be recursively chained.
*/
#[derive(Debug, Error, Diagnostic)]
#[error("{}", message)]
#[diagnostic(code(virshle::wrap::error))]
pub struct WrapError {
    pub message: String,
    #[diagnostic_source]
    pub origin: Report,
    #[help]
    pub help: String,
}

/**
A root cause error with no inner origin
*/
#[derive(Debug, Error, Diagnostic)]
#[error("{}", message)]
#[diagnostic(code(virshle::lib::error))]
pub struct LibError {
    pub message: String,
    #[help]
    pub help: String,
}

/**
A root cause error with no inner origin
*/
#[derive(Debug, Error, Diagnostic)]
#[error("{}", message)]
#[diagnostic(code(vishle::virt::error))]
pub struct VirtError {
    pub message: String,
    #[help]
    pub help: String,
    #[source]
    origin: virt::error::Error,
    pub code: u32,
}
impl VirtError {
    pub fn new(message: &str, help: &str, e: virt::error::Error) -> Self {
        Self {
            code: e.code().to_raw(),
            origin: e.clone(),
            message: message.to_owned(),
            help: help.to_owned(),
        }
    }
}
