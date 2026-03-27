use bon::builder;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use std::fmt::Debug;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use pipelight_exec::{Process, Status};

// Error Handling
use miette::Result;
use tracing::{debug, error, info, trace};
use virshle_error::{LibError, VirshleError};

use crate::Vm;

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
    filter += ",users=warn";
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
    filter += ",users=warn";
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

pub fn exec(cmd: &str) -> Result<String, VirshleError> {
    println!("\n");
    println!("-> {}\n", cmd);
    let mut proc = Process::new();
    proc.term().stdin(&cmd).run()?;
    let res: Option<String> = match proc.state.status {
        Some(Status::Succeeded) => proc.io.stdout.clone(),
        Some(Status::Failed) => proc.io.stderr.clone(),
        _ => Some("Command is in an unknown state.".to_owned()),
    };
    debug!(
        "Command Status: {:#?}\n I/O: {:#?}\n",
        proc.state.status, proc.io
    );
    Ok(res.unwrap_or("null".to_owned()))
}

// WARNING:
// External dependencies.
// Uses "socat" or "systemd-ssh-proxy"
#[builder(
    finish_fn = exec,
    on(String,into),
    on(Option<String>,into)
)]
pub async fn ssh(vm_name: &str, cmd: &str) -> Result<String, VirshleError> {
    // Set key path.
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../virshle_core/keys/user");
    let key = path.to_str().unwrap();

    // Systemd variant
    let cmd = format!(r#"ssh vm/{vm_name} "{cmd}""#);

    // TODO(): Socat variant
    // let vm = Vm::database().await?.one().name(vm_name).get().await?;
    // let path = vm.get_vsocket()?;
    // let id = "10".to_owned() + &vm.id.unwrap_or(1).to_string();
    // let name = vm.name;
    // let cmd = format!(
    //     r#"ssh -o ProxyCommand="echo -e \"CONNECT 22\\n\" | socat - UNIX-CONNECT:{path}" localhost "{cmd}""#
    // );

    let res = exec(&cmd)?;
    Ok(res)
}
