use std::sync::{Arc, Mutex, MutexGuard};

use axum::response::IntoResponse;
use compositor_pipeline::{error::InitPipelineError, pipeline};
use compositor_render::EventLoop;

use serde::Serialize;
use tokio::runtime::Runtime;

use crate::config::Config;

pub type Pipeline = compositor_pipeline::Pipeline;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Response {
    Ok {},
    RegisteredPort {
        port: Option<u16>,
    },
    RegisteredMp4 {
        video_duration_ms: Option<u64>,
        audio_duration_ms: Option<u64>,
    },
}

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
    }
}

#[derive(Clone)]
pub struct ApiState {
    pub pipeline: Arc<Mutex<Pipeline>>,
    pub config: Config,
}

impl ApiState {
    pub fn new(
        config: Config,
        runtime: Arc<Runtime>,
    ) -> Result<(ApiState, Arc<dyn EventLoop>), InitPipelineError> {
        let Config {
            queue_options,
            stream_fallback_timeout,
            web_renderer,
            force_gpu,
            download_root,
            output_sample_rate,
            stun_servers,
            required_wgpu_features,
            load_system_fonts,
            ..
        } = config.clone();
        let (pipeline, event_loop) = Pipeline::new(pipeline::Options {
            queue_options,
            stream_fallback_timeout,
            web_renderer,
            force_gpu,
            download_root,
            output_sample_rate,
            stun_servers,
            wgpu_features: required_wgpu_features,
            wgpu_ctx: None,
            load_system_fonts: Some(load_system_fonts),
            tokio_rt: Some(runtime),
        })?;
        Ok((
            ApiState {
                pipeline: Mutex::new(pipeline).into(),
                config,
            },
            event_loop,
        ))
    }

    pub(crate) fn pipeline(&self) -> MutexGuard<'_, Pipeline> {
        self.pipeline.lock().unwrap()
    }
}
