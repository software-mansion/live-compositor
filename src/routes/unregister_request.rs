use std::time::Duration;

use axum::extract::{Path, State};
use compositor_render::{error::ErrorStack, RegistryType};
use log::error;
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    state::{ApiState, Response},
    types::{InputId, OutputId, RendererId},
};

use super::Json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnregisterInput {
    /// Time in milliseconds when this request should be applied. Value `0` represents
    /// time of the start request.
    schedule_time_ms: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnregisterOutput {
    /// Time in milliseconds when this request should be applied. Value `0` represents
    /// time of the start request.
    schedule_time_ms: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnregisterRenderer {
    Shader { shader_id: RendererId },
    WebRenderer { instance_id: RendererId },
    Image { image_id: RendererId },
}

pub(super) async fn handle_input(
    State(api): State<ApiState>,
    Path(input_id): Path<InputId>,
    Json(request): Json<UnregisterInput>,
) -> Result<Response, ApiError> {
    match request.schedule_time_ms {
        Some(schedule_time_ms) => {
            let pipeline = api.pipeline.clone();
            let schedule_time = Duration::from_secs_f64(schedule_time_ms / 1000.0);
            api.pipeline().queue().schedule_event(
                schedule_time,
                Box::new(move || {
                    if let Err(err) = pipeline.lock().unwrap().unregister_input(&input_id.into()) {
                        error!(
                            "Error while running scheduled input unregister for pts {}ms: {}",
                            schedule_time.as_millis(),
                            ErrorStack::new(&err).into_string()
                        )
                    }
                }),
            );
        }
        None => {
            api.pipeline().unregister_input(&input_id.into())?;
        }
    }
    Ok(Response::Ok {})
}

pub(super) async fn handle_output(
    State(api): State<ApiState>,
    Path(output_id): Path<OutputId>,
    Json(request): Json<UnregisterOutput>,
) -> Result<Response, ApiError> {
    match request.schedule_time_ms {
        Some(schedule_time_ms) => {
            let pipeline = api.pipeline.clone();
            let schedule_time = Duration::from_secs_f64(schedule_time_ms / 1000.0);
            api.pipeline().queue().schedule_event(
                schedule_time,
                Box::new(move || {
                    if let Err(err) = pipeline
                        .lock()
                        .unwrap()
                        .unregister_output(&output_id.into())
                    {
                        error!(
                            "Error while running scheduled output unregister for pts {}ms: {}",
                            schedule_time.as_millis(),
                            ErrorStack::new(&err).into_string()
                        )
                    }
                }),
            );
        }
        None => {
            api.pipeline().unregister_output(&output_id.into())?;
        }
    }
    Ok(Response::Ok {})
}

pub(super) async fn handle_renderer(
    State(api): State<ApiState>,
    Json(request): Json<UnregisterRenderer>,
) -> Result<Response, ApiError> {
    match request {
        UnregisterRenderer::Shader { shader_id } => {
            api.pipeline()
                .unregister_renderer(&shader_id.into(), RegistryType::Shader)?;
        }
        UnregisterRenderer::WebRenderer { instance_id } => {
            api.pipeline()
                .unregister_renderer(&instance_id.into(), RegistryType::WebRenderer)?;
        }
        UnregisterRenderer::Image { image_id } => {
            api.pipeline()
                .unregister_renderer(&image_id.into(), RegistryType::Image)?;
        }
    }

    Ok(Response::Ok {})
}
