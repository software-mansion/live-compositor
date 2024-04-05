use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use compositor_pipeline::pipeline::{self};
use compositor_render::{
    error::{ErrorStack, InitPipelineError},
    EventLoop,
};

use log::error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    error::ApiError,
    types::{InputId, OutputId, RegisterRequest, RendererId, UpdateOutputRequest},
};

mod register_request;
mod unregister_request;

pub type Pipeline = compositor_pipeline::Pipeline;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Register(RegisterRequest),
    Unregister(UnregisterRequest),
    UpdateOutput(UpdateOutputRequest),
    Start,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "entity_type", rename_all = "snake_case")]
pub enum UnregisterRequest {
    InputStream {
        input_id: InputId,
        /// Time in milliseconds when this request should be applied. Value `0` represents
        /// time of the start request.
        schedule_time_ms: Option<f64>,
    },
    OutputStream {
        output_id: OutputId,
        /// Time in milliseconds when this request should be applied. Value `0` represents
        /// time of the start request.
        schedule_time_ms: Option<f64>,
    },
    Shader {
        shader_id: RendererId,
    },
    WebRenderer {
        instance_id: RendererId,
    },
    Image {
        image_id: RendererId,
    },
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Response {
    Ok {},
    RegisteredPort { port: u16 },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InputInfo {
    Rtp { id: InputId, port: u16 },
    Mp4 { id: InputId },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutputInfo {
    pub id: OutputId,
    pub port: u16,
    pub ip: Arc<str>,
}

#[derive(Clone)]
pub struct Api {
    pub pipeline: Arc<Mutex<Pipeline>>,
    pub config: Config,
}

impl Api {
    pub fn new(config: Config) -> Result<(Api, Arc<dyn EventLoop>), InitPipelineError> {
        let Config {
            queue_options,
            stream_fallback_timeout,
            web_renderer,
            force_gpu,
            download_root,
            output_sample_rate,
            ..
        } = config.clone();
        let (pipeline, event_loop) = Pipeline::new(pipeline::Options {
            queue_options,
            stream_fallback_timeout,
            web_renderer,
            force_gpu,
            download_root,
            output_sample_rate,
        })?;
        Ok((
            Api {
                pipeline: Mutex::new(pipeline).into(),
                config,
            },
            event_loop,
        ))
    }

    pub async fn handle_request(&self, request: Request) -> Result<Response, ApiError> {
        match request {
            Request::Register(register_request) => {
                register_request::handle_register_request(self, register_request).await
            }
            Request::Unregister(unregister_request) => {
                unregister_request::handle_unregister_request(self, unregister_request)
            }
            Request::Start => {
                Pipeline::start(&self.pipeline);
                Ok(Response::Ok {})
            }
            Request::UpdateOutput(update) => self.handle_scene_update(update),
        }
    }

    fn handle_scene_update(&self, update: UpdateOutputRequest) -> Result<Response, ApiError> {
        let output_id = update.output_id.into();
        let scene = match update.video {
            Some(component) => Some(component.try_into()?),
            None => None,
        };
        let audio = update.audio.map(|a| a.try_into()).transpose()?;

        match update.schedule_time_ms {
            Some(schedule_time_ms) => {
                let pipeline = self.pipeline.clone();
                let schedule_time = Duration::from_secs_f64(schedule_time_ms / 1000.0);
                self.pipeline().queue().schedule_event(
                    schedule_time,
                    Box::new(move || {
                        if let Err(err) = pipeline
                            .lock()
                            .unwrap()
                            .update_output(output_id, scene, audio)
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
            None => self.pipeline().update_output(output_id, scene, audio)?,
        };
        Ok(Response::Ok {})
    }

    fn pipeline(&self) -> MutexGuard<'_, Pipeline> {
        self.pipeline.lock().unwrap()
    }
}
