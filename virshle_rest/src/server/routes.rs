use crate::commons::{
    CreateManyVmArgs, CreateVmArgs, GetManyVmArgs, GetVmArgs, StartManyVmArgs, StartVmArgs,
};
use crate::server::{RestServer, Server};
use axum::{
    extract::{Extension, Path, Query},
    http::Request,
    middleware::map_response,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
// Global vars
use std::sync::{Arc, RwLock};

use pipelight_exec::Status;
use std::collections::HashMap;
use tower_http::trace::TraceLayer;
use virshle_core::{
    config::{Config, UserData, VmTemplate, VmTemplateTable},
    hypervisor::{
        vm::{Vm, VmInfo, VmTable},
        vmm::types::{VmInfoResponse, VmState},
    },
    peer::{HostInfo, NodeInfo, Peer},
};
// Error handling
use miette::Result;
use tracing::info;
use virshle_error::{LibError, VirshleError, WrapError};

impl RestServer {
    pub async fn build() -> Result<RestServer, VirshleError> {
        let res = RestServer {
            router: Self::make_api_v1().await?,
        };
        Ok(res)
    }

    /// Set server identity in response header.
    async fn set_header<B>(mut response: Response<B>) -> Response<B> {
        response
            .headers_mut()
            .insert("server", "Virshle API".parse().unwrap());
        response
    }
    /// Create Rest API routes.
    pub async fn make_api_v1() -> Result<Router, VirshleError> {
        // Virshle API
        // let server = Arc::new(RwLock::new(Server::new().build()?));
        // let server = Server::new().build()?;

        let api_v1_one: Router = Router::new()
            // Node
            // Check for the REST API availability
            .route(
                "/node/ping",
                get(async || {
                    let server = Server::new().build()?;
                    Result::<Json<()>, VirshleError>::Ok(Json(server.api()?.node().ping().await?))
                }),
            )
            .route(
                "/node/info",
                get(async || {
                    let server = Server::new().build()?;
                    Result::<Json<NodeInfo>, VirshleError>::Ok(Json(
                        server.api()?.node().info().await?,
                    ))
                }),
            )
            // Template
            .route(
                "/template/all",
                get(async || {
                    let server = Server::new().build()?;
                    Result::<Json<Vec<VmTemplate>>, VirshleError>::Ok(Json(
                        server.api()?.template().get_many().await?,
                    ))
                }),
            )
            .route(
                "/template/reclaim",
                get(async move |Json(params): Json<CreateVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<bool>, VirshleError>::Ok(Json(
                        server.clone().api()?.template().reclaim(params).await?,
                    ))
                }),
            )
            // Vm
            .route(
                "/vm/create",
                put(async move |Json(params): Json<CreateVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<VmTable>, VirshleError>::Ok(Json(
                        server
                            .api()?
                            .vm()
                            .create()
                            .one()
                            .maybe_template(params.template_name)
                            .maybe_user_data(params.user_data)
                            .exec()
                            .await?,
                    ))
                }),
            )
            .route(
                "/vm/start",
                put(async move |Json(params): Json<StartVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<VmTable>, VirshleError>::Ok(Json(
                        server
                            .api()?
                            .vm()
                            .start()
                            .one()
                            .maybe_id(params.id)
                            .maybe_name(params.name)
                            .maybe_uuid(params.uuid)
                            .maybe_user_data(params.user_data)
                            .exec()
                            .await?,
                    ))
                }),
            )
            .route(
                "/vm/shutdown",
                put(async move |Json(params): Json<GetVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<VmTable>, VirshleError>::Ok(Json(
                        server
                            .api()?
                            .vm()
                            .shutdown()
                            .one()
                            .maybe_id(params.id)
                            .maybe_name(params.name)
                            .maybe_uuid(params.uuid)
                            .exec()
                            .await?,
                    ))
                }),
            )
            .route(
                "/vm/delete",
                put(async move |Json(params): Json<GetVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<VmTable>, VirshleError>::Ok(Json(
                        server
                            .api()?
                            .vm()
                            .delete()
                            .one()
                            .maybe_id(params.id)
                            .maybe_name(params.name)
                            .maybe_uuid(params.uuid)
                            .exec()
                            .await?,
                    ))
                }),
            )
            .route(
                "/vm/info",
                post(async move |Json(params): Json<GetVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<VmTable>, VirshleError>::Ok(Json(
                        server
                            .api()?
                            .vm()
                            .get()
                            .one()
                            .maybe_id(params.id)
                            .maybe_name(params.name)
                            .maybe_uuid(params.uuid)
                            .exec()
                            .await?,
                    ))
                }),
            )
            .route(
                "/vm/get_vsock_path",
                post(async move |Json(params): Json<GetVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<String>, VirshleError>::Ok(Json(
                        server.api()?.vm().get_vsock_path(params).await?,
                    ))
                }),
            );
        // Virshle Bulk operation API
        let api_v1_many = Router::new()
            // Template
            .route(
                "/template/info.many",
                get(async || {
                    let server = Server::new().build()?;
                    Result::<Json<Vec<VmTemplateTable>>, VirshleError>::Ok(Json(
                        server.api()?.template().get_info_many().await?,
                    ))
                }),
            )
            // Vm
            .route(
                "/vm/get.many",
                post(async move |Json(params): Json<GetManyVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<Vec<VmTable>>, VirshleError>::Ok(Json(
                        server
                            .api()?
                            .vm()
                            .get()
                            .many()
                            .maybe_state(params.vm_state)
                            .maybe_account(params.account_uuid)
                            .exec()
                            .await?,
                    ))
                }),
            )
            .route(
                "/vm/create.many",
                put(async move |Json(params): Json<CreateManyVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<Vec<VmTable>>, VirshleError>::Ok(Json(
                        server
                            .api()?
                            .vm()
                            .create()
                            .many()
                            .maybe_template(params.template_name)
                            .maybe_user_data(params.user_data)
                            .maybe_n(params.ntimes)
                            .exec()
                            .await?,
                    ))
                }),
            )
            .route(
                "/vm/start.many",
                put(async move |Json(params): Json<StartManyVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<HashMap<Status, Vec<VmTable>>>, VirshleError>::Ok(Json(
                        server
                            .api()?
                            .vm()
                            .start()
                            .many()
                            .maybe_state(params.vm_state)
                            .maybe_account(params.account_uuid)
                            .exec()
                            .await?,
                    ))
                }),
            )
            .route(
                "/vm/shutdown.many",
                put(async move |Json(params): Json<GetManyVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<HashMap<Status, Vec<VmTable>>>, VirshleError>::Ok(Json(
                        server
                            .api()?
                            .vm()
                            .shutdown()
                            .many()
                            .maybe_state(params.vm_state)
                            .maybe_account(params.account_uuid)
                            .exec()
                            .await?,
                    ))
                }),
            )
            .route(
                "/vm/delete.many",
                put(async move |Json(params): Json<GetManyVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<HashMap<Status, Vec<VmTable>>>, VirshleError>::Ok(Json(
                        server
                            .api()?
                            .vm()
                            .delete()
                            .many()
                            .maybe_state(params.vm_state)
                            .maybe_account(params.account_uuid)
                            .exec()
                            .await?,
                    ))
                }),
            )
            .route(
                "/vm/info.many",
                post(async move |Json(params): Json<GetManyVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<Vec<VmTable>>, VirshleError>::Ok(Json(
                        server
                            .api()?
                            .vm()
                            .get()
                            .many()
                            .maybe_state(params.vm_state)
                            .maybe_account(params.account_uuid)
                            .exec()
                            .await?,
                    ))
                }),
            );
        // Cloud-hypervisor direct calls.
        let api_v1_ch = Router::new()
            // Vm
            .route(
                "/vm.info",
                post(async move |Json(params): Json<GetVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<VmInfoResponse>, VirshleError>::Ok(Json(
                        server.api()?.vm().get_ch_info(params).await?,
                    ))
                }),
            )
            .route(
                "/vm.info.raw",
                post(async move |Json(params): Json<GetVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<String>, VirshleError>::Ok(Json(
                        server.api()?.vm().get_raw_ch_info(params).await?,
                    ))
                }),
            )
            .route(
                "/vmm.ping",
                post(async move |Json(params): Json<GetVmArgs>| {
                    let server = Server::new().build()?;
                    Result::<Json<()>, VirshleError>::Ok(Json(
                        server.api()?.vm().ping_ch(params).await?,
                    ))
                }),
            );

        // Global routes
        let router = Router::new()
            .nest("/api/v1", api_v1_one)
            .nest("/api/v1", api_v1_many)
            .nest("/api/v1/ch", api_v1_ch)
            .layer(map_response(Self::set_header))
            .layer(TraceLayer::new_for_http());

        Ok(router)
    }
}
