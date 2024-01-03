use std::sync::Arc;

use compositor_pipeline::pipeline::{self};
use compositor_render::{EventLoop, RegistryType};
use crossbeam_channel::{bounded, Receiver};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tiny_http::StatusCode;

use crate::{
    error::ApiError,
    rtp_receiver::RtpReceiver,
    rtp_sender::RtpSender,
    types::{
        self, AudioInputId, AudioOutputId, InitOptions, RegisterRequest, RendererId, VideoInputId,
        VideoOutputId,
    },
};

mod register_request;

pub type Pipeline = compositor_pipeline::Pipeline<RtpReceiver, RtpSender>;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Init(InitOptions),
    Register(RegisterRequest),
    Unregister(UnregisterRequest),
    UpdateComposition(UpdateCompositionRequest),
    Query(QueryRequest),
    Start,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct UpdateCompositionRequest {
    pub video_outputs: Vec<types::VideoCompositionParams>,
    pub audio_outputs: Vec<types::AudioMixParams>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "entity_type", rename_all = "snake_case")]
pub enum UnregisterRequest {
    VideoInput { input_id: VideoInputId },
    AudioInput { input_id: AudioInputId },
    VideoOutput { output_id: VideoOutputId },
    AudioOutput { output_id: AudioOutputId },
    Shader { shader_id: RendererId },
    WebRenderer { instance_id: RendererId },
    Image { image_id: RendererId },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "query", rename_all = "snake_case")]
pub enum QueryRequest {
    WaitForNextFrame { input_id: VideoInputId },
    Inputs,
    Outputs,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged, deny_unknown_fields)]
pub enum Response {
    Ok {},
    Inputs { inputs: Vec<InputInfo> },
    Outputs { outputs: Vec<OutputInfo> },
    RegisteredPort(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Port {
    Range((u16, u16)),
    Exact(u16),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputInfo {
    pub id: VideoInputId,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutputInfo {
    pub id: VideoOutputId,
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
    pub fn new(opts: pipeline::Options) -> Result<(Api, EventLoop), ApiError> {
        let (pipeline, event_loop) = Pipeline::new(opts)?;
        Ok((Api { pipeline }, event_loop))
    }

    pub fn handle_request(&mut self, request: Request) -> Result<ResponseHandler, ApiError> {
        match request {
            Request::Init(_) => Err(ApiError::new(
                "COMPOSITOR_ALREADY_INITIALIZED",
                "Compositor was already initialized.".to_string(),
                StatusCode(400),
            )),
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
            Request::UpdateComposition(scene_spec) => {
                self.pipeline.update_scene(scene_spec.try_into()?)?;
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
                    .map(|(id, node)| InputInfo {
                        id: id.clone().into(),
                        port: node.port,
                    })
                    .collect();
                Ok(ResponseHandler::Response(Response::Inputs { inputs }))
            }
            QueryRequest::Outputs => {
                let outputs = self.pipeline.with_outputs(|iter| {
                    iter.map(|(id, output)| OutputInfo {
                        id: id.clone().into(),
                        port: output.port,
                        ip: output.ip.clone(),
                    })
                    .collect()
                });
                Ok(ResponseHandler::Response(Response::Outputs { outputs }))
            }
        }
    }

    fn handle_unregister_request(&mut self, request: UnregisterRequest) -> Result<(), ApiError> {
        match request {
            UnregisterRequest::VideoInput { input_id } => {
                Ok(self.pipeline.unregister_input(&input_id.into())?)
            }
            UnregisterRequest::VideoOutput { output_id } => {
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
            UnregisterRequest::AudioInput { .. } => todo!(),
            UnregisterRequest::AudioOutput { .. } => todo!(),
        }
    }
}
