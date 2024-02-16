use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use compositor_pipeline::pipeline::{self};
use compositor_render::{
    error::{ErrorStack, InitPipelineError},
    EventLoop,
};
use crossbeam_channel::{bounded, Receiver};

use log::error;
use serde::{Deserialize, Serialize};

use crate::{
    config::{config, Config},
    error::ApiError,
    types::{InputId, OutputId, OutputScene, RegisterRequest, RendererId},
};

mod register_request;
mod unregister_request;

pub type Pipeline = compositor_pipeline::Pipeline;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Register(RegisterRequest),
    Unregister(UnregisterRequest),
    UpdateScene(OutputScene),
    Query(QueryRequest),
    Start,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "entity_type", rename_all = "snake_case")]
pub enum UnregisterRequest {
    InputStream {
        input_id: InputId,
        /// Timestamp relative to start request when this request
        /// should be applied.
        schedule_time_ms: Option<f64>,
    },
    OutputStream {
        output_id: OutputId,
        /// Timestamp relative to start request when this request
        /// should be applied.
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

#[derive(Serialize, Deserialize)]
#[serde(tag = "query", rename_all = "snake_case")]
pub enum QueryRequest {
    WaitForNextFrame { input_id: InputId },
    Inputs,
    Outputs,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Response {
    Ok {},
    Inputs { inputs: Vec<InputInfo> },
    Outputs { outputs: Vec<OutputInfo> },
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

pub enum ResponseHandler {
    Response(Response),
    Ok,
    DeferredResponse(Receiver<Result<Response, ApiError>>),
}

pub struct Api {
    pipeline: Arc<Mutex<Pipeline>>,
}

impl Api {
    pub fn new() -> Result<(Api, Arc<dyn EventLoop>), InitPipelineError> {
        let Config {
            queue_options,
            stream_fallback_timeout,
            web_renderer,
            force_gpu,
            download_root,
            ..
        } = config();
        let (pipeline, event_loop) = Pipeline::new(pipeline::Options {
            queue_options: *queue_options,
            stream_fallback_timeout: *stream_fallback_timeout,
            web_renderer: *web_renderer,
            force_gpu: *force_gpu,
            download_root: download_root.clone(),
        })?;
        Ok((
            Api {
                pipeline: Mutex::new(pipeline).into(),
            },
            event_loop,
        ))
    }

    pub fn handle_request(&mut self, request: Request) -> Result<ResponseHandler, ApiError> {
        match request {
            Request::Register(register_request) => {
                register_request::handle_register_request(self, register_request)
            }
            Request::Unregister(unregister_request) => {
                unregister_request::handle_unregister_request(self, unregister_request)
            }
            Request::Start => {
                self.pipeline.lock().unwrap().start();
                Ok(ResponseHandler::Ok)
            }
            Request::UpdateScene(update) => self.handle_scene_update(update),
            Request::Query(query) => self.handle_query(query),
        }
    }

    fn handle_query(&self, query: QueryRequest) -> Result<ResponseHandler, ApiError> {
        match query {
            QueryRequest::WaitForNextFrame { input_id } => {
                let (sender, receiver) = bounded(1);
                self.pipeline().queue().subscribe_input_listener(
                    &input_id.into(),
                    Box::new(move || {
                        sender.send(Ok(Response::Ok {})).unwrap();
                    }),
                );
                Ok(ResponseHandler::DeferredResponse(receiver))
            }
            QueryRequest::Inputs => {
                let inputs = self
                    .pipeline()
                    .inputs()
                    .map(|(id, node)| match node.input {
                        pipeline::input::Input::Rtp(ref rtp) => InputInfo::Rtp {
                            id: id.clone().into(),
                            port: rtp.port,
                        },

                        pipeline::input::Input::Mp4(ref mp4) => InputInfo::Mp4 {
                            id: mp4.input_id.clone().into(),
                        },
                    })
                    .collect();
                Ok(ResponseHandler::Response(Response::Inputs { inputs }))
            }
            QueryRequest::Outputs => {
                let outputs = self.pipeline().with_outputs(|iter| {
                    iter.map(|(id, output)| match output.output {
                        pipeline::output::Output::Rtp(ref rtp) => OutputInfo {
                            id: id.clone().into(),
                            port: rtp.port,
                            ip: rtp.ip.clone(),
                        },
                    })
                    .collect()
                });
                Ok(ResponseHandler::Response(Response::Outputs { outputs }))
            }
        }
    }

    fn handle_scene_update(&self, update: OutputScene) -> Result<ResponseHandler, ApiError> {
        let output_id = update.output_id.into();
        let scene = update.scene.try_into()?;
        match update.schedule_time_ms {
            Some(schedule_time_ms) => {
                let pipeline = self.pipeline.clone();
                let schedule_time = Duration::from_secs_f64(schedule_time_ms / 1000.0);
                self.pipeline().queue().schedule_event(
                    schedule_time,
                    Box::new(move || {
                        if let Err(err) = pipeline.lock().unwrap().update_scene(output_id, scene) {
                            error!(
                                "Error while running scheduled output unregister for pts {}ms: {}",
                                schedule_time.as_millis(),
                                ErrorStack::new(&err).into_string()
                            )
                        }
                    }),
                );
            }
            None => self.pipeline().update_scene(output_id, scene)?,
        };
        Ok(ResponseHandler::Ok)
    }

    fn pipeline(&self) -> MutexGuard<'_, Pipeline> {
        self.pipeline.lock().unwrap()
    }
}
