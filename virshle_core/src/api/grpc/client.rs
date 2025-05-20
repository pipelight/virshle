use crate::config::NodeInfo;
use tonic::Request;

use crate::config::Node;
use std::collections::HashMap;

// Error handling
use log::{error, info, warn};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

use crate::api::grpc::server::get_info_client::GetInfoClient;
use crate::api::grpc::server::NodeService;

use super::server::GrpcServer;

pub struct GrpcClient;

impl GrpcClient {
    // pub async fn get_nodes_info() -> Result<()> {}
}
