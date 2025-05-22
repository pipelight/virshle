use crate::config::{NodeInfo, VirshleConfig};
use tonic::Request;

use crate::config::Node;
use crate::connection::{
    Connection, ConnectionHandle, ConnectionState, NodeConnection, SshConnection,
};
use std::collections::HashMap;

// Error handling
use log::{error, info, warn};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

use crate::api::grpc::server::get_info_client::GetInfoClient;
use crate::api::grpc::server::NodeService;

use super::server::get_info_client;
use super::server::GrpcServer;

pub struct GrpcClient;

impl GrpcClient {
    // pub async fn get_nodes_info() -> Result<()> {}
    pub async fn get_nodes_info() -> Result<(), VirshleError> {
        let config = VirshleConfig::get()?;
        let nodes = config.get_nodes()?;

        let mut node_info: HashMap<Node, (ConnectionState, Option<NodeInfo>)> = HashMap::new();
        for node in nodes {
            let mut conn = node.get_connection()?;
            match conn.open().await {
                Err(e) => {
                    error!("{}", e);
                    node_info.insert(node, (conn.get_state()?, None));
                }
                Ok(_) => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Node;
    use hyper_util::rt::TokioIo;
    use tokio::net::UnixStream;

    #[tokio::test]
    async fn test_send_grpc() -> Result<()> {
        // let conn = Node::default().get_connection()?;
        let conn = NodeConnection(Connection::SshConnection(SshConnection::new(
            "ssh://localhost/var/lib/virshle/virshle.sock",
        )?));

        let endpoint = "/grpc";

        match conn.0 {
            Connection::UnixConnection(connection) => {
                let mut client =
                    get_info_client::GetInfoClient::connect(connection.uri.to_string())
                        .await
                        .into_diagnostic()?;

                let res = client
                    .node_info(tonic::Request::new(()))
                    .await
                    .into_diagnostic()?;

                println!("{:#?}", res);

                // let request = tonic::Request::new(get_info_client::GetInfoClient::new(NodeService));
                // let response = unix_connection
                //     .open()
                //     .await?
                //     .send(endpoint, request)
                //     .await?;
                // return Ok(response);
            }
            Connection::SshConnection(mut connection) => {
                connection.open().await?;
                let channel = tonic::transport::Endpoint::try_from(
                    "unix://var/lib/virshle/virshle.sock",
                    // connection.uri.to_string()
                )
                .into_diagnostic()?
                .connect_with_connector(tower::service_fn(|_: tonic::transport::Uri| async {
                    let path = "unix://var/lib/virshle/virshle.sock"; // connection.uri.to_string()
                                                                      // Ok::<_, std::io::Error>(TokioIo::new(UnixStream::connect(path).await?))
                    Ok::<_, std::io::Error>(connection.handle.unwrap().connection)
                }))
                .await
                .into_diagnostic()?;

                let mut client =
                    get_info_client::GetInfoClient::connect(connection.uri.to_string())
                        .await
                        .into_diagnostic()?;

                let res = client
                    .node_info(tonic::Request::new(()))
                    .await
                    .into_diagnostic()?;

                println!("{:#?}", res);
            }
        };

        Ok(())
    }
}
