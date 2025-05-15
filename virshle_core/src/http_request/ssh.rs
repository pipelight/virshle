use super::{HttpSender, Response};
use crate::connection::SshConnection;

// Http
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response as HyperResponse, StatusCode};
use hyper_util::rt::TokioIo;

// Error Handling
use log::{info, trace};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError};

impl HttpSender for SshConnection {
    async fn send(
        &mut self,
        endpoint: &str,
        request: &Request<Full<Bytes>>,
    ) -> Result<Response, VirshleError> {
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
            let err = LibError::new("Connection has no handler.", "open connection first.");
            return Err(err.into());
        }
    }
}
