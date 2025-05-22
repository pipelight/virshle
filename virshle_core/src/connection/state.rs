use serde::{Deserialize, Serialize};

// Error Handling
use log::{debug, info, trace};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ConnectionState {
    /// Success: Connection established and daemon is up!
    DaemonUp,

    /// Uninitialized: Connection not established.
    #[default]
    Down,

    // Warning: Small error
    SshAuthError,

    // Error
    DaemonDown,
    SocketNotFound,
    /// Unknown network reason.
    Unreachable,
}
