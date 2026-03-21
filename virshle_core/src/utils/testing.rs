use bon::builder;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use std::fmt::Debug;
use std::fmt::Display;

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
) -> Result<(), VirshleError> {
    let level = verbosity.to_string().to_lowercase().to_owned();
    let mut filter: String = level.clone();
    match db {
        Some(true) => filter += &format!(",sea_orm={level},sqlx={level}"),
        Some(false) | None => filter += ",sea_orm=warn,sqlx=warn",
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
pub fn logger(verbosity: tracing::Level) -> Result<(), VirshleError> {
    std::env::set_var("VIRSHLE_LOG", tracing::Level::ERROR.to_string());
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Error)
        .init();
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
