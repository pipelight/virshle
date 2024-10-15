use std::path::Path;

use hyper::body::{Body, Bytes, Incoming};
use hyper::client::conn::http1::{handshake, SendRequest};
use hyper::{Request, Response as HyperResponse, StatusCode};

use http_body_util::{BodyExt, Full};
use hyper_util::rt::TokioIo;

use serde::{Deserialize, Serialize};
use serde_json::{from_slice, Value};

use tokio::net::UnixStream;
use tokio::spawn;
use tokio::task::JoinHandle;

// Error Handling
use log::{debug, info};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, WrapError};

pub struct Connection {
    sender: SendRequest<Full<Bytes>>,
    connection: JoinHandle<Result<(), hyper::Error>>,
}

impl Connection {
    pub async fn open(socket: &str) -> Result<Self, VirshleError> {
        let stream: TokioIo<UnixStream> =
            TokioIo::new(UnixStream::connect(Path::new(socket)).await?);

        let connection: Connection = match handshake(stream).await {
            Err(error) => return Err(error.into()),
            Ok((sender, connection)) => Self {
                sender,
                connection: spawn(async move { connection.await }),
            },
        };

        Ok(connection)
    }

    async fn execute(
        mut self,
        url: &str,
        request: Request<Full<Bytes>>,
    ) -> Result<Response, VirshleError> {
        let response: hyper::Response<Incoming> = self.sender.send_request(request).await?;

        let status: StatusCode = response.status();
        let response: Response = Response::new(url, response, self.connection);
        debug!("{:#?}", response);

        // if !status.is_success() {
        //     let message = format!("Status failed: {}", status);
        //     return Err(LibError::new(&message, "").into());
        // }
        //
        Ok(response)
    }

    pub async fn get(self, url: &str) -> Result<Response, VirshleError> {
        let request = Request::builder()
            .uri(url)
            .method("GET")
            .header("Host", "localhost")
            .body(Full::new(Bytes::new()));

        self.execute(url, request?).await
    }

    pub async fn post<T>(self, url: &str, body: Option<T>) -> Result<Response, VirshleError>
    where
        T: Serialize,
    {
        let request = Request::builder()
            .uri(url)
            .method("POST")
            .header("Host", "localhost")
            .header("Content-Type", "application/json");
        let request = match body {
            None => request.body(Full::new(Bytes::new())),
            Some(value) => request.body(Full::new(Bytes::from(
                serde_json::to_value(value).unwrap().to_string(),
            ))),
        };

        self.execute(url, request?).await
    }
    pub async fn put<T>(self, url: &str, body: Option<T>) -> Result<Response, VirshleError>
    where
        T: Serialize,
    {
        let request = Request::builder()
            .uri(url)
            .method("PUT")
            .header("Host", "localhost")
            .header("Content-Type", "application/json");

        let request = match body {
            None => request.body(Full::new(Bytes::new())),
            Some(value) => request.body(Full::new(Bytes::from(
                serde_json::to_value(value).unwrap().to_string(),
            ))),
        };

        self.execute(url, request?).await
    }
}
#[derive(Debug)]
pub struct Response {
    pub url: String,
    pub inner: HyperResponse<Incoming>,
    pub connection: JoinHandle<Result<(), hyper::Error>>,
}

impl Response {
    fn new(
        url: &str,
        response: HyperResponse<Incoming>,
        connection: JoinHandle<Result<(), hyper::Error>>,
    ) -> Self {
        Self {
            url: url.to_owned(),
            inner: response,
            connection,
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
        let status: StatusCode = self.inner.status();
        let data: Bytes = self.into_bytes().await?;
        let value: String = String::from_utf8(data.to_vec())?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::{IntoDiagnostic, Result};
    use std::path::PathBuf;
    use tokio::fs;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;
    use tokio::time::{sleep, Duration};

    /*
     * Create a socket and listens to incoming connections
     */
    async fn create_socket(socket: &str) -> Result<()> {
        let path = PathBuf::from(socket);
        let _ = tokio::fs::remove_file(&path).await;
        tokio::fs::create_dir_all(path.parent().unwrap())
            .await
            .unwrap();

        let listener = UnixListener::bind(socket).into_diagnostic()?;
        match listener.accept().await {
            Ok((mut socket, addr)) => {
                println!("Got a client: {:?} - {:?}", socket, addr);
                socket.write_all(b"hello world").await.into_diagnostic()?;
                let mut response = String::new();
                socket
                    .read_to_string(&mut response)
                    .await
                    .into_diagnostic()?;
                println!("{}", response);
            }
            Err(e) => println!("accept function failed: {:?}", e),
        }
        Ok(())
    }

    /*
     * Delete socket at specified path.
     * To be used after create_socket.
     */
    async fn remove_socket(socket: &str) -> Result<()> {
        fs::remove_file(socket).await.into_diagnostic()?;
        Ok(())
    }

    #[tokio::test]
    async fn try_send_req() -> Result<()> {
        let path = "./test.sock";
        // Create socket in other thread.
        spawn(async { create_socket(path).await.unwrap() });
        sleep(Duration::from_millis(100)).await;

        let conn = Connection::open(path).await?;
        conn.get("/vms").await?;

        remove_socket(path).await?;
        Ok(())
    }
}
