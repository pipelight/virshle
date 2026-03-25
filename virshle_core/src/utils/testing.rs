use bon::builder;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use std::fmt::Debug;
use std::fmt::Display;
use std::str::FromStr;

// Error Handling
use miette::Result;
use tracing::{debug, error, info, trace};
use virshle_error::{LibError, VirshleError};

/// Always print trace/logs inside test.
#[builder(finish_fn = set)]
pub fn tracer(
    verbosity: tracing::Level,
    /// Wether sqlite database logs should be enabled
    db: Option<bool>,
    /// Wether russh logs should be enabled
    ssh: Option<bool>,
    /// Wether http request/response logs should be enabled
    http: Option<bool>,
) -> Result<(), VirshleError> {
    // Get tracing level from command line or fallback to function args.
    let env_var: Option<String> = std::env::var("VIRSHLE_TRACING").ok();
    let level: String = match env_var {
        Some(v) => v,
        None => verbosity.to_string().to_lowercase().to_owned(),
    };
    // Set crate filter
    let mut filter: String = level.clone();
    filter += "users=warn";
    match db {
        Some(true) => filter += &format!(",sea_orm={level},sqlx={level}"),
        Some(false) | None => filter += ",sea_orm=warn,sqlx=warn",
    };
    match http {
        Some(true) => filter += &format!(",tower_http={level},mio={level}"),
        Some(false) | None => filter += ",tower_http=warn,mio=warn",
    };
    match ssh {
        Some(true) => filter += &format!(",russh={level}"),
        Some(false) | None => filter += ",russh=error",
    };

    let builder = FmtSubscriber::builder()
        .with_max_level(verbosity)
        .with_env_filter(EnvFilter::try_new(filter).unwrap());

    let builder = builder.pretty();
    // let builder = builder.compact();
    let subscriber = builder.finish();
    tracing::subscriber::set_global_default(subscriber).ok();

    Ok(())
}
/// Always print logs inside test.
// Mainly used in tests for pretty printing tables.
// A lower debug level means reasonable data amount to print out.
#[builder(finish_fn = set)]
pub fn logger(
    verbosity: tracing::Level,
    /// Wether sqlite database logs should be enabled
    db: Option<bool>,
    /// Wether russh logs should be enabled
    ssh: Option<bool>,
    /// Wether http request/response logs should be enabled
    http: Option<bool>,
) -> Result<(), VirshleError> {
    let env_var: Option<String> = std::env::var("VIRSHLE_LOG").ok();
    let level: String = match env_var {
        Some(v) => v,
        None => verbosity.to_string().to_lowercase().to_owned(),
    };

    // Set crate filter
    let mut filter: String = level.clone();
    filter += "users=warn";
    match db {
        Some(true) => filter += &format!(",sea_orm={level},sqlx={level}"),
        Some(false) | None => filter += ",sea_orm=warn,sqlx=warn",
    };
    match http {
        Some(true) => filter += &format!(",tower_http={level},mio={level}"),
        Some(false) | None => filter += ",tower_http=warn,mio=warn",
    };
    match ssh {
        Some(true) => filter += &format!(",russh={level}"),
        Some(false) | None => filter += ",russh=error",
    };
    env_logger::Builder::new()
        .parse_filters(&filter)
        .try_init()
        .ok();
    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn unwind<T, E>(result: Result<T, E>) -> Result<()>
where
    T: Debug,
    E: Display + Debug,
{
    if result.is_err() {
        error!("{:#?}", result.unwrap());
    } else {
        debug!("{:#?}", result.unwrap());
    }
    Ok(())
}
