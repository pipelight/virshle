pub mod response;

use crate::connection::{Connection, ConnectionHandle};
use crate::connection::{Stream, Streamable};
use response::Response;

// Http
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::client::conn::http1; // {handshake, SendRequest};
use hyper::{Request, Response as HyperResponse, StatusCode};
use hyper_util::rt::TokioIo;

use serde::{Deserialize, Serialize};
use std::time;
use tokio::time::timeout;

// Socket
use tokio::spawn;
use tokio::task::JoinHandle;

// Serde
use serde::de::DeserializeOwned;

use std::future::Future;
// Error Handling
use log::{debug, error, info, trace};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub struct RestClient<'a> {
    pub connection: &'a mut Connection,
    handle: Option<StreamHandle>,
}

pub struct StreamHandle {
    sender: http1::SendRequest<Full<Bytes>>,
    connection: JoinHandle<Result<(), hyper::Error>>,
}

pub trait Rest {
    /*
     * Open connection to:
     * - a unix socket,
     * - a unix socket through ssh on a remote
     *
     * Do the http1 or http2 handshake.
     *
     * And return a gRpc or REST or cli.
     */
    fn open(&mut self) -> impl Future<Output = Result<&mut Self, VirshleError>> + Send;

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

impl<'a> Rest for RestClient<'a> {
    async fn open(&mut self) -> Result<&mut Self, VirshleError> {
        if self.handle.is_none() {
            match self.connection.open().await {
                Ok(stream) => {
                    // Test handshake
                    let mut handle = handshake(stream).await?;

                    // A running cloud-hypervisor process can be clunky
                    // and wait forever after a handshake so we test response
                    // duration.

                    // Test endpoint response
                    let request = Request::builder()
                        .uri("/")
                        .method("GET")
                        .header("Host", "localhost")
                        .body(Full::new(Bytes::new()))?;

                    // Timeout reponse and return succesfully if a response is sent,
                    // Wether it is a succesful response or an error message.
                    let time: u64 = 1000;
                    let response = handle.sender.send_request(request.to_owned());
                    let response = timeout(time::Duration::from_millis(time), response)
                        .await
                        .map_err(|e| {
                            LibError::builder()
                                .msg(&e.to_string())
                                .help(&format!("Request timeout {time}ms reached."))
                                .build()
                        })?;

                    self.handle = Some(handle);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(self)
    }
    async fn send(
        &mut self,
        endpoint: &str,
        request: &Request<Full<Bytes>>,
    ) -> Result<Response, VirshleError> {
        trace!("{:#?}", request);

        // Ensure connection is open and has a stream handle.
        self.open().await?;

        if let Some(handle) = &mut self.handle {
            let send: Result<HyperResponse<Incoming>, _> =
                handle.sender.send_request(request.to_owned()).await;
            match send {
                Ok(response) => {
                    let status: StatusCode = response.status();
                    let response: Response = Response::new(endpoint, response);
                    trace!("{:#?}", response);

                    if !status.is_success() {
                        let status = status.to_string();
                        error!("{}", status);
                    }

                    Ok(response)
                }
                Err(e) => {
                    error!("{:#?}", e);
                    Err(e.into())
                }
            }
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
            .body(Full::new(Bytes::new()));

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

pub async fn handshake(stream: Stream) -> Result<StreamHandle, VirshleError> {
    match stream {
        Stream::Ssh(v) => {
            let v = TokioIo::new(v);
            match http1::handshake(v).await {
                Ok((sender, connection)) => {
                    let handle = StreamHandle {
                        sender,
                        connection: spawn(async move { connection.await }),
                    };
                    trace!("http1 handshake succeeded");
                    Ok(handle)
                }
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
            }
        }
        Stream::Socket(v) => {
            let v = TokioIo::new(v);
            match http1::handshake(v).await {
                Ok((sender, connection)) => {
                    let handle = StreamHandle {
                        sender,
                        connection: spawn(async move { connection.await }),
                    };
                    trace!("http1 handshake succeeded");
                    Ok(handle)
                }
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
            }
        }
    }
}

impl<'a> From<&'a mut Connection> for RestClient<'a> {
    fn from(value: &'a mut Connection) -> Self {
        let cli = RestClient {
            connection: value,
            handle: None,
        };
        return cli;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_make_rest_cli() -> Result<()> {
        Ok(())
    }
}
