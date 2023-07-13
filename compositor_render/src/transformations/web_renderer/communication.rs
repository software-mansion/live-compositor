use std::{
    io::{self, Read},
    num::ParseIntError, time::Instant,
};

use compositor_common::scene::Resolution;
use serde::{Deserialize, Serialize};

pub struct HttpClient {
    port: u16,
}

impl HttpClient {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn render_request(
        &self,
        url: String,
        resolution: Resolution,
    ) -> Result<Vec<u8>, HttpClientError> {
        let start = Instant::now();
        let resp = ureq::post(&format!("http://localhost:{}/render", self.port))
            .send_json(RenderRequest { url, resolution })?;

        // println!("ELAPSED {}", (Instant::now() - start).as_millis());
        
        let body_len: usize = resp
            .header("Content-Length")
            .ok_or(HttpClientError::NoContentLength)?
            .parse()?;

        let mut frame = Vec::with_capacity(body_len);
        resp.into_reader().read_to_end(&mut frame)?;

        // Ok(frame)

        // let resp = ureq::post(&format!("http://localhost:{}/render", self.port))
        //     .send_json(RenderRequest { url, resolution })?
        //     .into_json::<RenderResponse>()?;
        println!("ELAPSED {}", (Instant::now() - start).as_millis());

        // Ok(frame)

        Ok(Vec::new())
    }
}

#[derive(Serialize)]
pub struct RenderRequest {
    url: String,
    resolution: Resolution,
}

#[derive(Deserialize)]
pub struct RenderResponse {
    frame: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum HttpClientError {
    #[error("failed to make a request")]
    HttpRequestFailed(#[from] ureq::Error),

    #[error("Content-Length header not provided")]
    NoContentLength,

    #[error("invalid Content-Length header")]
    InvalidContentLength(#[from] ParseIntError),

    #[error("failed to deserialize response")]
    DeserializeError(#[from] io::Error),
}
