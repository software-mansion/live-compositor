use axum::extract::{Path, State};
use compositor_pipeline::pipeline::Port;
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    routes::Json,
    state::{Pipeline, Response},
    types::{
        ImageSpec, InputId, Mp4, OutputId, RegisterOutputRequest, RtpInputStream, ShaderSpec,
        WebRendererSpec,
    },
};

use super::ApiState;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RegisterInput {
    RtpStream(RtpInputStream),
    Mp4(Mp4),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RegisterOutput {
    RtpStream(RegisterOutputRequest),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RegisterRenderer {
    Shader(ShaderSpec),
    WebRenderer(WebRendererSpec),
    Image(ImageSpec),
}

pub(super) async fn handle_input(
    State(api): State<ApiState>,
    Path(input_id): Path<InputId>,
    Json(request): Json<RegisterInput>,
) -> Result<Response, ApiError> {
    let api = api.clone();
    tokio::task::spawn_blocking(move || {
        let response = match request {
            RegisterInput::RtpStream(rtp) => {
                Pipeline::register_input(&api.pipeline, input_id.into(), rtp.try_into()?)?
            }
            RegisterInput::Mp4(mp4) => {
                Pipeline::register_input(&api.pipeline, input_id.into(), mp4.try_into()?)?
            }
        };
        match response {
            Some(Port(port)) => Ok(Response::RegisteredPort { port }),
            None => Ok(Response::Ok {}),
        }
    })
    .await
    // `unwrap()` panics only when the task panicked or `response.abort()` was called
    .unwrap()
}

pub(super) async fn handle_output(
    State(api): State<ApiState>,
    Path(output_id): Path<OutputId>,
    Json(request): Json<RegisterOutput>,
) -> Result<Response, ApiError> {
    let api = api.clone();
    tokio::task::spawn_blocking(move || {
        let response = match request {
            RegisterOutput::RtpStream(rtp) => {
                Pipeline::register_output(&mut api.pipeline(), output_id.into(), rtp.try_into()?)?
            }
        };
        match response {
            Some(Port(port)) => Ok(Response::RegisteredPort { port }),
            None => Ok(Response::Ok {}),
        }
    })
    .await
    .unwrap()
}

pub(super) async fn handle_renderer(
    State(api): State<ApiState>,
    Json(request): Json<RegisterRenderer>,
) -> Result<Response, ApiError> {
    let api = api.clone();
    tokio::task::spawn_blocking(move || {
        let request = match request {
            RegisterRenderer::Shader(shader) => shader.try_into()?,
            RegisterRenderer::WebRenderer(web) => web.try_into()?,
            RegisterRenderer::Image(image) => image.try_into()?,
        };
        Pipeline::register_renderer(&api.pipeline, request)?;
        Ok(Response::Ok {})
    })
    .await
    .unwrap()
}
