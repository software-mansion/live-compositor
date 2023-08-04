use bytes::Bytes;
use compositor_common::scene::Resolution;
use reqwest::{blocking::Client, StatusCode};
use serde::{Deserialize, Serialize};

use super::SessionId;

pub struct ElectronApiClient {
    port: u16,
    client: Client,
}

impl ElectronApiClient {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            client: Client::new(),
        }
    }

    pub fn new_session(
        &self,
        url: &str,
        resolution: Resolution,
    ) -> Result<SessionId, ElectronApiError> {
        let resp: NewSessionResponse = self
            .client
            .post(self.get_endpoint("new_session"))
            .json(&NewSessionRequest { url, resolution })
            .send()?
            .json()?;

        Ok(resp.session_id)
    }

    pub fn get_frame(&self, session_id: &SessionId) -> Result<Bytes, ElectronApiError> {
        let resp = self
            .client
            .post(self.get_endpoint("get_frame"))
            .json(&GetFrameRequest { session_id })
            .send()?;

        if resp.status() != StatusCode::OK {
            let err_resp: ErrorResponse = resp.json()?;
            return Err(ElectronApiError::ApiError(err_resp.error));
        }

        Ok(resp.bytes()?)
    }

    fn get_endpoint(&self, route: &str) -> String {
        format!("http://127.0.0.1:{}/{}", self.port, route)
    }
}

#[derive(Serialize)]
struct NewSessionRequest<'a> {
    url: &'a str,
    resolution: Resolution,
}

#[derive(Deserialize)]
struct NewSessionResponse {
    session_id: SessionId,
}

#[derive(Serialize)]
struct GetFrameRequest<'a> {
    session_id: &'a SessionId,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ElectronApiError {
    #[error("failed to make a request")]
    HttpRequestFailed(#[from] reqwest::Error),

    #[error("web renderer api returned an error")]
    ApiError(String),
}
