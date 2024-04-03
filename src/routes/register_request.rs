use axum::extract::State;
use compositor_pipeline::pipeline::{Port, RegisterInputOptions};
use compositor_render::InputId;

use crate::{
    api::{Pipeline, Response},
    error::ApiError,
    routes::Json,
    types::{ImageSpec, Mp4, RegisterOutputRequest, RtpInputStream, ShaderSpec, WebRendererSpec},
};

use super::Api;

fn handle_register_input(
    api: &Api,
    input_id: InputId,
    register_options: RegisterInputOptions,
) -> Result<Response, ApiError> {
    match Pipeline::register_input(&api.pipeline, input_id, register_options)? {
        Some(Port(port)) => Ok(Response::RegisteredPort { port }),
        None => Ok(Response::Ok {}),
    }
}

pub(super) async fn handle_rtp_input_stream(
    State(api): State<Api>,
    Json(request): Json<RtpInputStream>,
) -> Result<Response, ApiError> {
    let api = api.clone();
    tokio::task::spawn_blocking(move || {
        let (input_id, register_options) = request.try_into()?;
        handle_register_input(&api, input_id, register_options)
    })
    .await
    // `unwrap()` panics only when the task panicked or `response.abort()` was called
    .unwrap()
}

pub(super) async fn handle_mp4(
    State(api): State<Api>,
    Json(request): Json<Mp4>,
) -> Result<Response, ApiError> {
    let api = api.clone();
    tokio::task::spawn_blocking(move || {
        let (input_id, register_options) = request.try_into()?;
        handle_register_input(&api, input_id, register_options)
    })
    .await
    .unwrap()
}

pub(super) async fn handle_rtp_output_stream(
    State(api): State<Api>,
    Json(request): Json<RegisterOutputRequest>,
) -> Result<Response, ApiError> {
    let api = api.clone();
    tokio::task::spawn_blocking(move || {
        match api.pipeline().register_output(request.try_into()?)? {
            Some(Port(port)) => Ok(Response::RegisteredPort { port }),
            None => Ok(Response::Ok {}),
        }
    })
    .await
    .unwrap()
}

pub(super) async fn handle_shader(
    State(api): State<Api>,
    Json(request): Json<ShaderSpec>,
) -> Result<Response, ApiError> {
    let api = api.clone();
    tokio::task::spawn_blocking(move || {
        Pipeline::register_renderer(&api.pipeline, request.try_into()?)?;
        Ok(Response::Ok {})
    })
    .await
    .unwrap()
}

pub(super) async fn handle_web_renderer(
    State(api): State<Api>,
    Json(request): Json<WebRendererSpec>,
) -> Result<Response, ApiError> {
    let api = api.clone();
    tokio::task::spawn_blocking(move || {
        Pipeline::register_renderer(&api.pipeline, request.try_into()?)?;
        Ok(Response::Ok {})
    })
    .await
    .unwrap()
}

pub(super) async fn handle_image(
    State(api): State<Api>,
    Json(request): Json<ImageSpec>,
) -> Result<Response, ApiError> {
    let api = api.clone();
    tokio::task::spawn_blocking(move || {
        Pipeline::register_renderer(&api.pipeline, request.try_into()?)?;
        Ok(Response::Ok {})
    })
    .await
    .unwrap()
}
