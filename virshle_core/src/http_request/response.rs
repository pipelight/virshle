use http_body_util::BodyExt;
use hyper::body::{Bytes, Incoming};
use hyper::{Request, Response as HyperResponse, StatusCode};

// Serde
use convert_case::{Case, Casing};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{from_slice, Value};

// Error Handling
use log::{debug, info, trace};
use miette::{Error, IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError, VirshleErrorResponse, WrapError};

#[derive(Debug)]
pub struct Response {
    pub url: String,
    pub inner: HyperResponse<Incoming>,
}

/*
* Convenient methods to easily convert and troubleshoot a reponse.
*/
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
    pub async fn to_value<T>(self) -> Result<T, VirshleError>
    where
        T: DeserializeOwned + std::fmt::Debug,
    {
        let status: StatusCode = self.inner.status();
        if status.is_success() {
            let string = &self.to_string().await?;
            let value: T = serde_json::from_str(string)?;
            trace!("Body {{\n{:#?}\n}}", value);
            Ok(value)
        } else {
            let string = &self.to_string().await?;

            let value: VirshleErrorResponse =
                match serde_json::from_str::<VirshleErrorResponse>(string) {
                    Ok(mut v) => {
                        v.help = format!("{}\n", status) + &v.help;
                        v
                    }
                    Err(e) => VirshleErrorResponse {
                        message: "UnknownErrorType".to_owned(),
                        help: format!("{}\n", status) + string,
                    },
                };

            Err(LibError::builder()
                .msg(&value.message)
                .help(&value.help)
                .build()
                .into())
        }
    }
}

/*
* Convert a request into a readable string.
* Used to ease debugging.
*/
pub fn request_to_string<T>(req: &Request<T>) -> Result<String, VirshleError>
where
    T: Serialize,
{
    let mut string = "".to_owned();

    string.push_str(&format!(
        "{} {} {:?}\n",
        req.method(),
        req.uri(),
        req.version()
    ));

    for (key, value) in req.headers() {
        let key = key.to_string().to_case(Case::Title);
        let value = value.to_str().unwrap();
        string.push_str(&format!("{key}: {value}\n",));
    }

    let body: Value = serde_json::to_value(req.body().to_owned())?;
    match body {
        Value::Null => {}
        _ => {
            let body: String = serde_json::to_string(req.body().to_owned())?;
            string.push_str(&format!("\n{}\n", body));
        }
    };
    Ok(string)
}
