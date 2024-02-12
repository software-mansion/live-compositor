use std::sync::Arc;

use compositor_pipeline::pipeline::{self};
use compositor_render::{error::InitPipelineError, EventLoop, RegistryType};
use crossbeam_channel::{bounded, Receiver};

use serde::{Deserialize, Serialize};

use crate::{
    config::{config, Config},
    error::ApiError,
    types::{InputId, OutputId, RegisterRequest, RendererId, UpdateOutputRequest},
};

mod register_request;

pub type Pipeline = compositor_pipeline::Pipeline;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Register(RegisterRequest),
    Unregister(UnregisterRequest),
    UpdateOutput(UpdateOutputRequest),
    Query(QueryRequest),
    Start,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "entity_type", rename_all = "snake_case")]
pub enum UnregisterRequest {
    InputStream { input_id: InputId },
    OutputStream { output_id: OutputId },
    Shader { shader_id: RendererId },
    WebRenderer { instance_id: RendererId },
    Image { image_id: RendererId },
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
    pipeline: Pipeline,
}

impl Api {
    pub fn new() -> Result<(Api, Arc<dyn EventLoop>), InitPipelineError> {
        let Config {
            framerate,
            stream_fallback_timeout,
            web_renderer,
            force_gpu,
            download_root,
            ..
        } = config();
        let (pipeline, event_loop) = Pipeline::new(pipeline::Options {
            framerate: *framerate,
            stream_fallback_timeout: *stream_fallback_timeout,
            web_renderer: *web_renderer,
            force_gpu: *force_gpu,
            download_root: download_root.clone(),
        })?;
        Ok((Api { pipeline }, event_loop))
    }

    pub fn handle_request(&mut self, request: Request) -> Result<ResponseHandler, ApiError> {
        match request {
            Request::Register(register_request) => {
                match register_request::handle_register_request(self, register_request)? {
                    Some(response) => Ok(response),
                    None => Ok(ResponseHandler::Ok),
                }
            }
            Request::Unregister(unregister_request) => {
                self.handle_unregister_request(unregister_request)?;
                Ok(ResponseHandler::Ok)
            }
            Request::Start => {
                self.pipeline.start();
                Ok(ResponseHandler::Ok)
            }
            Request::UpdateOutput(output_spec) => {
                let root_component = match output_spec.video {
                    Some(root_component) => Some(root_component.try_into()?),
                    None => None,
                };
                let audio_composition = output_spec.audio.map(|audio| audio.into());

                self.pipeline.update_scene(
                    output_spec.output_id.into(),
                    root_component,
                    audio_composition,
                )?;

                Ok(ResponseHandler::Ok)
            }
            Request::Query(query) => self.handle_query(query),
        }
    }

    fn handle_query(&self, query: QueryRequest) -> Result<ResponseHandler, ApiError> {
        match query {
            QueryRequest::WaitForNextFrame { input_id } => {
                let (sender, receiver) = bounded(1);
                self.pipeline.queue().subscribe_input_listener(
                    input_id.into(),
                    Box::new(move || {
                        sender.send(Ok(Response::Ok {})).unwrap();
                    }),
                );
                Ok(ResponseHandler::DeferredResponse(receiver))
            }
            QueryRequest::Inputs => {
                let inputs = self
                    .pipeline
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
                let outputs = self.pipeline.with_outputs(|iter| {
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

    fn handle_unregister_request(&mut self, request: UnregisterRequest) -> Result<(), ApiError> {
        match request {
            UnregisterRequest::InputStream { input_id } => {
                Ok(self.pipeline.unregister_input(&input_id.into())?)
            }
            UnregisterRequest::OutputStream { output_id } => {
                Ok(self.pipeline.unregister_output(&output_id.into())?)
            }
            UnregisterRequest::Shader { shader_id } => Ok(self
                .pipeline
                .unregister_renderer(&shader_id.into(), RegistryType::Shader)?),
            UnregisterRequest::WebRenderer { instance_id } => Ok(self
                .pipeline
                .unregister_renderer(&instance_id.into(), RegistryType::WebRenderer)?),
            UnregisterRequest::Image { image_id } => Ok(self
                .pipeline
                .unregister_renderer(&image_id.into(), RegistryType::Image)?),
        }
    }
}
