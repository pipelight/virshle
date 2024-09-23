pub mod clean;
pub mod create;
pub mod net;
pub mod secret;
pub mod vm;

// Reexport
pub use clean::clean;
pub use create::create;
pub use create::ResourceType;
pub use net::Net;
pub use secret::Secret;
pub use vm::Vm;

use virt::connect::Connect;

// Error Handling
use crate::error::{VirshleError, WrapError};
use log::{info, log_enabled, Level};
use miette::{IntoDiagnostic, Result};

pub fn connect() -> Result<Connect, VirshleError> {
    // let conn = Connect::open(Some("test:///default")).into_diagnostic()?;
    let res = Connect::open(Some("qemu:///system"))?;
    Ok(res)
}
