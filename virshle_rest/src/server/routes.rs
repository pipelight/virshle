use crate::commons::{
    CreateManyVmArgs, CreateVmArgs, GetManyVmArgs, GetVmArgs, StartManyVmArgs, StartVmArgs,
};
use crate::server::Server;
use axum::{
    extract::{Extension, Path, Query},
    http::Request,
    middleware::map_response,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use pipelight_exec::Status;
use std::collections::HashMap;
use tower_http::trace::TraceLayer;
use virshle_core::{
    config::{UserData, VmTemplate, VmTemplateTable},
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

impl Server {
    pub async fn make_router(&self) -> Result<Router, VirshleError> {
        // Virshle API
        let api_v1 = Router::new()
            // Node
            // Check for the REST API availability
            .route(
                "/node/ping",
                get(async || {
                    let methods = Server::methods()?;
                    Result::<Json<()>, VirshleError>::Ok(Json(methods.clone().node().ping().await?))
                }),
            )
            .route(
                "/node/info",
                get(async || {
                    let methods = Server::methods()?;
                    Result::<Json<NodeInfo>, VirshleError>::Ok(Json(methods.node().info().await?))
                }),
            )
            // Template
            .route(
                "/template/all",
                get(async || {
                    let methods = Server::methods()?;
                    Result::<Json<Vec<VmTemplate>>, VirshleError>::Ok(Json(
                        methods.template().get_many().await?,
                    ))
                }),
            )
            .route(
                "/template/reclaim",
                get(async move |Json(params): Json<CreateVmArgs>| {
                    let methods = Server::methods()?;
                    Result::<Json<bool>, VirshleError>::Ok(Json(
                        methods.template().reclaim(params).await?,
                    ))
                }),
            )
            // Vm
            .route(
                "/vm/create",
                put(async move |Json(params): Json<CreateVmArgs>| {
                    let methods = Server::methods()?;
                    Result::<Json<VmTable>, VirshleError>::Ok(Json(
                        methods
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
                    let methods = Server::methods()?;
                    Result::<Json<VmTable>, VirshleError>::Ok(Json(
                        methods
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
                    let methods = Server::methods()?;
                    Result::<Json<VmTable>, VirshleError>::Ok(Json(
                        methods
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
                    let methods = Server::methods()?;
                    Result::<Json<VmTable>, VirshleError>::Ok(Json(
                        methods
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
                    let methods = Server::methods()?;
                    Result::<Json<VmTable>, VirshleError>::Ok(Json(
                        methods
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
                    let methods = Server::methods()?;
                    Result::<Json<String>, VirshleError>::Ok(Json(
                        methods.vm().get_vsock_path(params).await?,
                    ))
                }),
            );

        // Virshle Bulk API
        let api_v1_bulk = Router::new()
            // Template
            .route(
                "/template/info.many",
                get(async || {
                    let methods = Server::methods()?;
                    Result::<Json<Vec<VmTemplateTable>>, VirshleError>::Ok(Json(
                        methods.template().get_info_many().await?,
                    ))
                }),
            )
            // Vm
            .route(
                "/vm/get.many",
                post(async move |Json(params): Json<GetManyVmArgs>| {
                    let methods = Server::methods()?;
                    Result::<Json<Vec<VmTable>>, VirshleError>::Ok(Json(
                        methods
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
                    let methods = Server::methods()?;
                    Result::<Json<Vec<VmTable>>, VirshleError>::Ok(Json(
                        methods
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
                    let methods = Server::methods()?;
                    Result::<Json<HashMap<Status, Vec<VmTable>>>, VirshleError>::Ok(Json(
                        methods
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
                    let methods = Server::methods()?;
                    Result::<Json<HashMap<Status, Vec<VmTable>>>, VirshleError>::Ok(Json(
                        methods
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
                    let methods = Server::methods()?;
                    Result::<Json<HashMap<Status, Vec<VmTable>>>, VirshleError>::Ok(Json(
                        methods
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
                    let methods = Server::methods()?;
                    Result::<Json<Vec<VmTable>>, VirshleError>::Ok(Json(
                        methods
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
                    let methods = Server::methods()?;
                    Result::<Json<VmInfoResponse>, VirshleError>::Ok(Json(
                        methods.vm().get_ch_info(params).await?,
                    ))
                }),
            )
            .route(
                "/vm.info.raw",
                post(async move |Json(params): Json<GetVmArgs>| {
                    let methods = Server::methods()?;
                    Result::<Json<String>, VirshleError>::Ok(Json(
                        methods.vm().get_raw_ch_info(params).await?,
                    ))
                }),
            )
            .route(
                "/vmm.ping",
                post(async move |Json(params): Json<GetVmArgs>| {
                    let methods = Server::methods()?;
                    Result::<Json<()>, VirshleError>::Ok(Json(methods.vm().ping_ch(params).await?))
                }),
            );

        let app = Router::new()
            .nest("/api/v1", api_v1)
            .nest("/api/v1", api_v1_bulk)
            .nest("/api/v1/ch", api_v1_ch)
            .layer(map_response(Self::set_header))
            .layer(TraceLayer::new_for_http());

        Ok(app)
    }

    async fn set_header<B>(mut response: Response<B>) -> Response<B> {
        response
            .headers_mut()
            .insert("server", "Virshle API".parse().unwrap());
        response
    }

    /// Run REST api.
    pub async fn run() -> Result<(), VirshleError> {
        let server = Server::default();
        let app = server.make_router().await?;
        let socket_path = Server::get_socket()?;

        info!("Server listening on socket {}", &socket_path);
        tokio_scoped::scope(|s| {
            s.spawn(async {
                let listener = Server::make_socket(&socket_path).await.unwrap();
                let _ = axum::serve(listener, app.clone()).await;
            });
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_run() -> Result<()> {
        Server::run().await?;
        Ok(())
    }
}
