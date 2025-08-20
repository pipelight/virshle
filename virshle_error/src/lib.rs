use bon::bon;
use miette::{Diagnostic, Report};
pub use pipelight_error::{CastError, JsonError, PipelightError, TomlError};

// Http
use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use thiserror::Error;
use tracing::error;

#[derive(Debug, Error, Diagnostic)]
pub enum VirshleError {
    ////////////////////////////////
    // Lib native errors
    #[error(transparent)]
    #[diagnostic(transparent)]
    WrapError(#[from] WrapError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    LibError(#[from] LibError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    ConnectionError(#[from] ConnectionError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    VirshleErrorResponse(#[from] VirshleErrorResponse),

    ////////////////////////////////
    // Type convertion
    #[error(transparent)]
    #[diagnostic(code(parse::error))]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error(transparent)]
    #[diagnostic(code(parse::error))]
    ParseError(#[from] url::ParseError),

    #[error(transparent)]
    #[diagnostic(code(serde::error))]
    SerdeError(#[from] serde_json::Error),

    #[error(transparent)]
    #[diagnostic(code(virshle::bat::error))]
    StrumError(#[from] strum::ParseError),

    #[error(transparent)]
    #[diagnostic(code(virshle::bat::error))]
    BatError(#[from] bat::error::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    CastError(#[from] CastError),

    #[error(transparent)]
    #[diagnostic(code(virshle::io::error))]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    #[diagnostic(code(virshle::io::error))]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    #[diagnostic(code(virshle::io::error))]
    UuidError(#[from] uuid::Error),

    #[error(transparent)]
    #[diagnostic(code(virshle::csv::error))]
    CsvError(#[from] csv::Error),

    ////////////////////////////////
    // Process execution
    #[error(transparent)]
    #[diagnostic(transparent)]
    PipelightError(#[from] PipelightError),

    // Database
    #[error(transparent)]
    #[diagnostic(code(sea_orm::error))]
    SeaOrmError(#[from] sea_orm::DbErr),

    // Http
    #[error(transparent)]
    #[diagnostic(code(hyper::error))]
    HyprError(#[from] hyper::Error),

    #[error(transparent)]
    #[diagnostic(code(hyper::error))]
    HyprHttpError(#[from] hyper::http::Error),

    // Env var error
    // Mainly use to get ssh_auth_agent socket.
    #[error(transparent)]
    #[diagnostic(code(env::error))]
    EnvError(#[from] std::env::VarError),

    // Env var error
    // Mainly use to get ssh_auth_agent socket.
    #[error(transparent)]
    #[diagnostic(code(future::error))]
    JoinError(#[from] tokio::task::JoinError),
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

#[bon]
impl WrapError {
    #[builder]
    pub fn new(msg: &str, help: &str, origin: Report) -> Self {
        Self {
            message: msg.to_owned(),
            help: help.to_owned(),
            origin,
        }
    }
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
#[bon]
impl LibError {
    #[builder]
    pub fn new(msg: &str, help: &str) -> Self {
        Self {
            message: msg.to_owned(),
            help: help.to_owned(),
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum ConnectionError {
    #[error("socket not found")]
    SocketNotFound,
    #[error("daemon is down")]
    DaemonDown,

    // Ssh
    #[error("failed ssh authentication")]
    SshAuthError,

    #[error(transparent)]
    #[diagnostic(code(ssh::error))]
    RusshError(#[from] russh::Error),

    #[error(transparent)]
    #[diagnostic(code(ssh::error))]
    SshKeyError(#[from] russh::keys::Error),

    #[error(transparent)]
    #[diagnostic(code(ssh::error))]
    SshAgentError(#[from] russh::AgentAuthError),
}

#[derive(Debug, Clone, Serialize, Deserialize, Error, Diagnostic)]
#[error("{}", message)]
#[diagnostic(code(api::error))]
pub struct VirshleErrorResponse {
    pub message: String,
    pub help: String,
}
impl IntoResponse for VirshleError {
    fn into_response(self) -> Response<Body> {
        let message = self.to_string();
        error!("{}", message);

        let status = StatusCode::INTERNAL_SERVER_ERROR;
        let mut err = VirshleErrorResponse {
            message,
            help: "".to_owned(),
        };
        if let Some(origin) = self.diagnostic_source() {
            err.help = origin.to_string();
        }

        let body = Body::from(serde_json::to_value(err).unwrap().to_string());
        let res = Response::builder().status(status).body(body).unwrap();
        return res;
    }
}
