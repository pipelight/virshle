mod response;
mod socket;
mod ssh;

use crate::connection::{Connection, ConnectionHandle, NodeConnection};
use response::Response;

// Http
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::client::conn::http1; // {handshake, SendRequest};
use hyper::{Request, Response as HyperResponse, StatusCode};
use hyper_util::rt::TokioIo; // {handshake};

use serde::{Deserialize, Serialize};

// Socket
use tokio::spawn;
use tokio::task::JoinHandle;

// Serde
use serde::de::DeserializeOwned;

use std::future::Future;
// Error Handling
use log::{debug, info, trace};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub struct Client<'a> {
    connection: &'a Connection,
    handle: Option<StreamHandle>,
}

pub struct StreamHandle {
    pub sender: http1::SendRequest<Full<Bytes>>,
    pub connection: JoinHandle<Result<(), hyper::Error>>,
}

pub trait RestClient {
    /*
     * Open connection to:
     * - a unix socket,
     * - a unix socket through ssh on a remote
     * And return a gRpc or REST or cli.
     */
    fn open(&mut self) -> impl Future<Output = Result<&mut Self, VirshleError>> + Send;
    /*
     * Close connection
     */
    fn close(&self) -> impl Future<Output = Result<(), VirshleError>> + Send;

    /*
     * Send the http request.
     * Internally used by get(), post() and put() methods.
     */
    fn send(
        &mut self,
        endpoint: &str,
        request: &Request<Full<Bytes>>,
    ) -> impl Future<Output = Result<Response, VirshleError>> + Send;

    /*
     * Send an http GET request to socket.
     * Arguments:
     * - path: the url enpoint (ex:"/vm/info")
     */
    fn get(&mut self, enpoint: &str)
        -> impl Future<Output = Result<Response, VirshleError>> + Send;
    /*
     * Send an http POST request to socket.
     * Arguments:
     * - path: the url enpoint (ex:"/vm/info")
     */
    fn post<T>(
        &mut self,
        enpoint: &str,
        body: Option<T>,
    ) -> impl Future<Output = Result<Response, VirshleError>> + Send
    where
        T: Serialize + Send;
    /*
     * Send an http PUT request to socket.
     * Arguments:
     * - path: the url enpoint (ex:"/vm/info")
     */
    fn put<T>(
        &mut self,
        enpoint: &str,
        body: Option<T>,
    ) -> impl Future<Output = Result<Response, VirshleError>> + Send
    where
        T: Serialize + Send;
}

impl RestClient for Client {
    async fn open(&self) -> Result<(), VirshleError> {
        if let Some(stream) = self.connection.open().await?.get_stream() {
            match http1::handshake(stream).await {
                Err(e) => {
                    let message = "Counldn't reach rest api (http1 handshake error)";
                    let help = "Is a rest api running on the socket?";
                    let err = WrapError::builder()
                        .msg(&message)
                        .help(&help)
                        .origin(Error::from_err(e))
                        .build();
                    return Err(err.into());
                }
                Ok((sender, connection)) => {
                    info!("Connected to socket at {}", self.uri);
                    self.handle = Some(StreamHandle {
                        sender,
                        connection: spawn(async move { connection.await }),
                    });
                }
            };
        }
        Ok(())
    }
    async fn send(
        &mut self,
        endpoint: &str,
        request: &Request<Full<Bytes>>,
    ) -> Result<Response, VirshleError> {
        debug!("{:#?}", request);

        if let Some(handle) = &mut self.handle {
            let response: HyperResponse<Incoming> =
                handle.sender.send_request(request.to_owned()).await?;

            let status: StatusCode = response.status();
            let response: Response = Response::new(endpoint, response);
            trace!("{:#?}", response);

            // if !status.is_success() {
            //     let message = format!("Status failed: {}", status);
            //     return Err(LibError::new(&message, "").into());
            // }

            Ok(response)
        } else {
            let err = LibError::builder()
                .msg("Connection has no handler.")
                .help("open connection first.")
                .build();
            return Err(err.into());
        }
    }

    async fn get(&mut self, endpoint: &str) -> Result<Response, VirshleError> {
        let request = Request::builder()
            .uri(endpoint)
            .method("GET")
            .header("Host", "localhost")
            .body(Full::new(Bytes::new()))?;

        self.send(endpoint, &request?).await
    }

    async fn post<T>(&mut self, endpoint: &str, body: Option<T>) -> Result<Response, VirshleError>
    where
        T: Serialize,
    {
        let request = Request::builder()
            .uri(endpoint)
            .method("POST")
            .header("Host", "localhost")
            .header("Content-Type", "application/json");

        let request = match body {
            None => request.body(Full::new(Bytes::new())),
            Some(value) => request.body(Full::new(Bytes::from(
                serde_json::to_value(value).unwrap().to_string(),
            ))),
        };
        self.send(endpoint, &request?).await
    }

    async fn put<T>(&mut self, endpoint: &str, body: Option<T>) -> Result<Response, VirshleError>
    where
        T: Serialize,
    {
        let request = Request::builder()
            .uri(endpoint)
            .method("PUT")
            .header("Host", "localhost")
            .header("Content-Type", "application/json");

        let request = match body {
            None => request.body(Full::new(Bytes::new())),
            Some(value) => request.body(Full::new(Bytes::from(
                serde_json::to_value(value).unwrap().to_string(),
            ))),
        };
        self.send(endpoint, &request?).await
    }
}

// trait GRpcSender {
//     /*
//      * Send the http2 request.
//      */
//     fn grpc_req<T>(
//         &mut self,
//         endpoint: &str,
//         request: &tonic::Request<T>,
//     ) -> impl Future<Output = Result<tonic::Response<T>, VirshleError>> + Send
//     where
//         T: Serialize + Send;
// }
//
// impl GRpcSender for Connection {
//     async fn grpc_req<T>(
//         &mut self,
//         enpoint: &str,
//         request: &tonic::Request<T>,
//     ) -> Result<tonic::Response<T>, VirshleError>
//     where
//         T: Serialize + Send,
//     {
//         Ok(())
//     }
// }

// impl Connection {
//     async fn grpc_req<T>(
//         &mut self,
//         enpoint: &str,
//         request: &tonic::Request<T>,
//     ) -> Result<tonic::Response<T>, VirshleError>
//     where
//         T: Serialize + Send,
//     {
//         match self.open()
//         Ok(())
//     }
// }
//
