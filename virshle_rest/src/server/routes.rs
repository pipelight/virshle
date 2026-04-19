use crate::commons::{
    CreateManyVmArgs, CreateVmArgs, GetManyVmArgs, GetVmArgs, StartManyVmArgs, StartVmArgs,
};
use crate::server::Server;
use axum::{
    extract::{Extension, Path, Query, State},
    http::Request,
    middleware::map_response,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};

// Global vars
use std::sync::Arc;
use tokio::sync::RwLock;

use indexmap::IndexMap;
use pipelight_exec::Status;
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

impl Server {
    /// Set server identity in response header.
    async fn set_header<B>(mut response: Response<B>) -> Response<B> {
        response
            .headers_mut()
            .insert("server", "Virshle API".parse().unwrap());
        response
    }
    /// Create Rest API routes.
    pub async fn make_router(&mut self) -> Result<(), VirshleError> {
        // Virshle API
        let api_v1: Router = Router::new()
            // Node
            // Check for the REST API availability
            .route(
                "/node/ping",
                get(async |State(server): State<Server>| {
                    Result::<Json<()>, VirshleError>::Ok(Json(server.api()?.node().ping().await?))
                }),
            )
            .route(
                "/node/info",
                get(async |State(server): State<Server>| {
                    Result::<Json<NodeInfo>, VirshleError>::Ok(Json(
                        server.api()?.node().info().await?,
                    ))
                }),
            )
            .route(
                "/node/id",
                get(async |State(server): State<Server>| {
                    Result::<Json<String>, VirshleError>::Ok(Json(
                        server.api()?.node().did().await?,
                    ))
                }),
            )
            // Template
            .route(
                "/template/all",
                get(async |State(server): State<Server>| {
                    Result::<Json<IndexMap<String, VmTemplate>>, VirshleError>::Ok(Json(
                        server.api()?.template().get_many().await?,
                    ))
                }),
            )
            .route(
                "/template/info.many",
                get(async |State(server): State<Server>| {
                    Result::<Json<IndexMap<String, VmTemplateTable>>, VirshleError>::Ok(Json(
                        server.api()?.template().get_info_many().await?,
                    ))
                }),
            )
            .route(
                "/template/reclaim",
                get(
                    async move |State(server): State<Server>, Json(params): Json<CreateVmArgs>| {
                        Result::<Json<bool>, VirshleError>::Ok(Json(
                            server.clone().api()?.template().reclaim(params).await?,
                        ))
                    },
                ),
            )
            // Vm
            .route(
                "/vm/info",
                post(
                    async move |State(server): State<Server>, Json(params): Json<GetVmArgs>| {
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
                    },
                ),
            )
            .route(
                "/vm/info.many",
                post(
                    async move |State(server): State<Server>, Json(params): Json<GetManyVmArgs>| {
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
                    },
                ),
            )
            .route(
                "/vm/create",
                put(
                    async move |State(server): State<Server>, Json(params): Json<CreateVmArgs>| {
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
                    },
                ),
            )
            .route(
                "/vm/create.many",
                put(
                    async move |State(server): State<Server>,
                                Json(params): Json<CreateManyVmArgs>| {
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
                    },
                ),
            )
            .route(
                "/vm/start",
                put(
                    async move |State(server): State<Server>, Json(params): Json<StartVmArgs>| {
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
                                .maybe_attach(params.attach)
                                .maybe_fresh(params.fresh)
                                .exec()
                                .await?,
                        ))
                    },
                ),
            )
            .route(
                "/vm/provision-ch",
                put(
                    async move |State(server): State<Server>, Json(params): Json<StartVmArgs>| {
                        Result::<Json<VmTable>, VirshleError>::Ok(Json(
                            server
                                .api()?
                                .vm()
                                .start()
                                .provision_ch()
                                .maybe_id(params.id)
                                .maybe_name(params.name)
                                .maybe_uuid(params.uuid)
                                .exec()
                                .await?,
                        ))
                    },
                ),
            )
            .route(
                "/vm/create-init-resources",
                put(
                    async move |State(server): State<Server>, Json(params): Json<StartVmArgs>| {
                        Result::<Json<VmTable>, VirshleError>::Ok(Json(
                            server
                                .api()?
                                .vm()
                                .start()
                                .create_init_resources()
                                .maybe_id(params.id)
                                .maybe_name(params.name)
                                .maybe_uuid(params.uuid)
                                .maybe_user_data(params.user_data)
                                .exec()
                                .await?,
                        ))
                    },
                ),
            )
            .route(
                "/vm/start.many",
                put(
                    async move |State(server): State<Server>,
                                Json(params): Json<StartManyVmArgs>| {
                        Result::<Json<IndexMap<Status, Vec<VmTable>>>, VirshleError>::Ok(Json(
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
                    },
                ),
            )
            .route(
                "/vm/shutdown",
                put(
                    async move |State(server): State<Server>, Json(params): Json<GetVmArgs>| {
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
                    },
                ),
            )
            .route(
                "/vm/shutdown.many",
                put(
                    async move |State(server): State<Server>, Json(params): Json<GetManyVmArgs>| {
                        Result::<Json<IndexMap<Status, Vec<VmTable>>>, VirshleError>::Ok(Json(
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
                    },
                ),
            )
            .route(
                "/vm/delete",
                put(
                    async move |State(server): State<Server>, Json(params): Json<GetVmArgs>| {
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
                    },
                ),
            )
            .route(
                "/vm/delete.many",
                put(
                    async move |State(server): State<Server>, Json(params): Json<GetManyVmArgs>| {
                        Result::<Json<IndexMap<Status, Vec<VmTable>>>, VirshleError>::Ok(Json(
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
                    },
                ),
            )
            .route(
                "/vm/get_vsock_path",
                post(
                    async move |State(server): State<Server>, Json(params): Json<GetVmArgs>| {
                        Result::<Json<String>, VirshleError>::Ok(Json(
                            server.api()?.vm().get_vsock_path(params).await?,
                        ))
                    },
                ),
            )
            .with_state(self.clone());
        // Cloud-hypervisor direct calls.
        let api_v1_ch = Router::new()
            // Vm
            .route(
                "/vm.info",
                post(
                    async move |State(server): State<Server>, Json(params): Json<GetVmArgs>| {
                        Result::<Json<VmInfoResponse>, VirshleError>::Ok(Json(
                            server.api()?.vm().get_ch_info(params).await?,
                        ))
                    },
                ),
            )
            .route(
                "/vm.info.raw",
                post(
                    async move |State(server): State<Server>, Json(params): Json<GetVmArgs>| {
                        Result::<Json<String>, VirshleError>::Ok(Json(
                            server.api()?.vm().get_raw_ch_info(params).await?,
                        ))
                    },
                ),
            )
            .route(
                "/vmm.ping",
                post(
                    async move |State(server): State<Server>, Json(params): Json<GetVmArgs>| {
                        Result::<Json<()>, VirshleError>::Ok(Json(
                            server.api()?.vm().ping_ch(params).await?,
                        ))
                    },
                ),
            )
            .with_state(self.clone());

        // Global routes
        let router = Router::new()
            .nest("/api/v1", api_v1)
            .nest("/api/v1/ch", api_v1_ch)
            .layer(map_response(Self::set_header))
            .layer(TraceLayer::new_for_http());

        self.router = router;

        Ok(())
    }
}
