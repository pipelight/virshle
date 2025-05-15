use super::{HttpRequest, HttpSender, Response};
use crate::connection::{Connection, NodeConnection};

// Http
use http_body_util::{BodyExt, Full};
use hyper::body::{Body, Bytes, Incoming};
use hyper::{Request, StatusCode};
use serde::{Deserialize, Serialize};

// Error Handling
use log::info;
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

impl HttpSender for NodeConnection {
    async fn send(
        &mut self,
        endpoint: &str,
        request: &Request<Full<Bytes>>,
    ) -> Result<Response, VirshleError> {
        self.0.send(endpoint, request).await
    }
}

impl HttpRequest for NodeConnection {
    async fn get(&mut self, enpoint: &str) -> Result<Response, VirshleError> {
        self.0.get(enpoint).await
    }
    async fn post<T>(&mut self, enpoint: &str, body: Option<T>) -> Result<Response, VirshleError>
    where
        T: Serialize + Send,
    {
        self.0.post(enpoint, body).await
    }
    async fn put<T>(&mut self, enpoint: &str, body: Option<T>) -> Result<Response, VirshleError>
    where
        T: Serialize + Send,
    {
        self.0.put(enpoint, body).await
    }
}
