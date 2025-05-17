mod node;
mod socket;
mod ssh;
mod vm;

use crate::connection::{Connection, ConnectionHandle, NodeConnection};

// Http
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::client::conn::http1::{handshake, SendRequest};
use hyper::{Request, Response as HyperResponse, StatusCode};
use hyper_util::rt::TokioIo;

use serde::{Deserialize, Serialize};
use serde_json::{from_slice, Value};

// Socket
use tokio::spawn;
use tokio::task::JoinHandle;

// Serde
use serde::de::DeserializeOwned;

use std::future::Future;
// Error Handling
use log::{debug, info};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub trait HttpSender {
    /*
     * Send the http request.
     * Internally used by get(), post() and put() methods.
     */
    fn send(
        &mut self,
        endpoint: &str,
        request: &Request<Full<Bytes>>,
    ) -> impl Future<Output = Result<Response, VirshleError>> + Send;
}

pub trait HttpRequest {
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

#[derive(Debug)]
pub struct Response {
    pub url: String,
    pub inner: HyperResponse<Incoming>,
}
impl Response {
    pub fn new(url: &str, response: HyperResponse<Incoming>) -> Self {
        Self {
            url: url.to_owned(),
            inner: response,
        }
    }
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }
    pub async fn into_bytes(self) -> Result<Bytes, VirshleError> {
        let data = self.inner.into_body().collect().await?;
        let data = data.to_bytes();
        Ok(data)
    }
    pub async fn to_string(self) -> Result<String, VirshleError> {
        let data: Bytes = self.into_bytes().await?;
        let value: String = String::from_utf8(data.to_vec())?;
        Ok(value)
    }
    pub async fn to_value<T: DeserializeOwned>(self) -> Result<T, VirshleError> {
        let status: StatusCode = self.inner.status();
        if status.is_success() {
            let value: T = serde_json::from_str(&self.to_string().await?)?;
            Ok(value)
        } else {
            let message = "Http response error";
            let help = format!("{}", status);
            Err(LibError::builder().msg(message).help(&help).build().into())
        }
    }
}

impl HttpSender for Connection {
    async fn send(
        &mut self,
        endpoint: &str,
        request: &Request<Full<Bytes>>,
    ) -> Result<Response, VirshleError> {
        debug!("{:#?}", request);
        match self {
            Connection::SshConnection(ssh_connection) => {
                let response = ssh_connection.open().await?.send(endpoint, request).await?;
                return Ok(response);
            }
            Connection::UnixConnection(unix_connection) => {
                let response = unix_connection
                    .open()
                    .await?
                    .send(endpoint, request)
                    .await?;
                return Ok(response);
            }
        };
    }
}

impl HttpRequest for Connection {
    async fn get(&mut self, endpoint: &str) -> Result<Response, VirshleError> {
        let request = Request::builder()
            .uri(endpoint)
            .method("GET")
            .header("Host", "localhost")
            .body(Full::new(Bytes::new()))?;

        match self {
            Connection::SshConnection(ssh_connection) => {
                ssh_connection.send(endpoint, &request).await
            }
            Connection::UnixConnection(unix_connection) => {
                unix_connection.send(endpoint, &request).await
            }
        }
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
