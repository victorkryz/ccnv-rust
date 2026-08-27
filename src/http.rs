//! Blocking HTTP transport used by the currency-rate service.

use std::time::Duration;

use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use thiserror::Error;

#[derive(Debug)]
pub struct HttpResponse {
    pub status: StatusCode,
    pub body: String,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("{message}")]
pub struct HttpError {
    message: String,
}

impl HttpError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<reqwest::Error> for HttpError {
    fn from(error: reqwest::Error) -> Self {
        Self::new(error.to_string())
    }
}

pub trait HttpTransport {
    fn get(&self, url: &str) -> Result<HttpResponse, HttpError>;
}

#[derive(Debug)]
pub struct ReqwestTransport {
    client: Client,
}

impl ReqwestTransport {
    pub fn new() -> Result<Self, HttpError> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .redirect(Policy::limited(5))
            .user_agent(concat!("ccnv/", env!("CARGO_PKG_VERSION")))
            .build()?;

        Ok(Self { client })
    }
}

impl HttpTransport for ReqwestTransport {
    fn get(&self, url: &str) -> Result<HttpResponse, HttpError> {
        let response = self.client.get(url).send()?;
        let status = response.status();
        let body = response.text()?;

        Ok(HttpResponse { status, body })
    }
}
