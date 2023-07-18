use std::{io, num::ParseIntError};

use bytes::Bytes;
use compositor_common::scene::Resolution;
use reqwest::blocking::Client;
use serde::Serialize;

pub struct HttpClient {
    port: u16,
    client: Client,
}

impl HttpClient {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            client: Client::new(),
        }
    }

    pub fn get_frame(&self, url: String, resolution: Resolution) -> Result<Bytes, HttpClientError> {
        let resp = self
            .client
            .post(format!("http://localhost:{}/get_frame", self.port))
            .json(&RenderRequest { url, resolution })
            .send()?;

        Ok(resp.bytes()?)
    }
}

#[derive(Serialize)]
pub struct RenderRequest {
    url: String,
    resolution: Resolution,
}

#[derive(Debug, thiserror::Error)]
pub enum HttpClientError {
    #[error("failed to make a request")]
    HttpRequestFailed(#[from] reqwest::Error),

    #[error("Content-Length header not provided")]
    NoContentLength,

    #[error("invalid Content-Length header")]
    InvalidContentLength(#[from] ParseIntError),

    #[error("failed to deserialize response")]
    DeserializeError(#[from] io::Error),
}
