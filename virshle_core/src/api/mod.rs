use axum::{
    extract::connect_info::{self, ConnectInfo},
    http::Request,
    routing::get,
    Router,
};
use hyper::body::Incoming;
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server,
};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::{unix::UCred, UnixListener, UnixStream};
use tower::Service;

use std::path::PathBuf;

// Hypervisor
use crate::cloud_hypervisor::Vm;

// Error handling
use log::info;
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub struct Api;

impl Api {
    pub async fn run() -> Result<(), VirshleError> {
        // build our application with a single route
        let app = Router::new()
            .route(
                "/vm/list",
                get(|| async {
                    serde_json::to_string(&Vm::get_all().await.unwrap());
                }),
            )
            .route(
                "/vm/create/:size",
                get(|| async {
                    // serde_json::to_string(&Vm::set().await.unwrap());
                }),
            )
            .route(
                "/node",
                get(|| async {
                    serde_json::to_string(&Vm::get_all().await.unwrap());
                }),
            )
            .route("/", get(|| async { "Hello, World!" }));

        let path = "/var/lib/virshle/virshle.socket";
        let path = PathBuf::from(path);

        // Ensure clean socket
        let _ = tokio::fs::remove_file(&path).await;
        tokio::fs::create_dir_all(path.parent().unwrap())
            .await
            .unwrap();

        let listener = UnixListener::bind("/var/lib/virshle/virshle.socket")?;
        let mut make_service = app.into_make_service_with_connect_info::<UdsConnectInfo>();

        // See https://github.com/tokio-rs/axum/blob/main/examples/serve-with-hyper/src/main.rs for
        // more details about this setup
        loop {
            let (socket, _remote_addr) = listener.accept().await.unwrap();

            let tower_service = unwrap_infallible(make_service.call(&socket).await);

            tokio::spawn(async move {
                let socket = TokioIo::new(socket);

                let hyper_service =
                    hyper::service::service_fn(move |request: Request<Incoming>| {
                        tower_service.clone().call(request)
                    });

                if let Err(err) = server::conn::auto::Builder::new(TokioExecutor::new())
                    .serve_connection_with_upgrades(socket, hyper_service)
                    .await
                {
                    eprintln!("failed to serve connection: {err:#}");
                }
            });
        }
    }
}

#[derive(Clone, Debug)]
struct UdsConnectInfo {
    peer_addr: Arc<tokio::net::unix::SocketAddr>,
    peer_cred: UCred,
}

impl connect_info::Connected<&UnixStream> for UdsConnectInfo {
    fn connect_info(target: &UnixStream) -> Self {
        let peer_addr = target.peer_addr().unwrap();
        let peer_cred = target.peer_cred().unwrap();

        Self {
            peer_addr: Arc::new(peer_addr),
            peer_cred,
        }
    }
}
fn unwrap_infallible<T>(result: Result<T, Infallible>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => match err {},
    }
}
