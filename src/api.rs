use std::sync::Arc;

use anyhow::{anyhow, Result};
use compositor_common::{
    scene::{InputId, OutputId, Resolution, SceneSpec},
    transformation::{TransformationRegistryKey, TransformationSpec},
};
use compositor_pipeline::pipeline;
use crossbeam_channel::{bounded, Receiver};
use serde::{Deserialize, Serialize};

use crate::{
    rtp_receiver::{self, RtpReceiver},
    rtp_sender::{self, EncoderSettings, RtpSender},
};

pub type Pipeline = compositor_pipeline::Pipeline<RtpReceiver, RtpSender>;

#[derive(Serialize, Deserialize)]
pub struct RegisterInputRequest {
    pub id: InputId,
    pub port: u16,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterOutputRequest {
    pub id: OutputId,
    pub port: u16,
    pub ip: Arc<str>,
    pub resolution: Resolution,
    pub encoder_settings: EncoderSettings,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Init(pipeline::Options),
    RegisterInput(RegisterInputRequest),
    UnregisterInput {
        id: InputId,
    },
    RegisterOutput(RegisterOutputRequest),
    UnregisterOutput {
        id: OutputId,
    },
    RegisterTransformation {
        key: TransformationRegistryKey,
        transform: TransformationSpec,
    },
    UpdateScene(Arc<SceneSpec>),
    Query(QueryRequest),
    Start,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "query", rename_all = "snake_case")]
pub enum QueryRequest {
    WaitForNextFrame { input_id: InputId },
    Scene,
    Inputs,
    Outputs,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Response {
    Ok {},
    Scene(Arc<SceneSpec>),
    Inputs { inputs: Vec<InputInfo> },
    Outputs { outputs: Vec<OutputInfo> },
}

#[derive(Serialize, Deserialize)]
pub struct InputInfo {
    pub id: InputId,
    pub port: u16,
}

#[derive(Serialize, Deserialize)]
pub struct OutputInfo {
    pub id: OutputId,
    pub port: u16,
    pub ip: Arc<str>,
}

pub enum ResponseHandler {
    Response(Response),
    Ok,
    DeferredResponse(Receiver<Result<Response>>),
}

pub struct Api {
    pipeline: Pipeline,
}

impl Api {
    pub fn new(opts: pipeline::Options) -> Result<Api> {
        Ok(Api {
            pipeline: Pipeline::new(opts)?,
        })
    }

    pub fn handle_request(&mut self, request: Request) -> Result<ResponseHandler> {
        match request {
            Request::Init(_) => Err(anyhow!("Video compositor is already initialized.")),
            Request::RegisterInput(request) => {
                self.register_input(request).map(|_| ResponseHandler::Ok)
            }
            Request::UnregisterInput { id } => {
                self.pipeline.unregister_input(&id)?;
                Ok(ResponseHandler::Ok)
            }
            Request::RegisterOutput(request) => {
                self.register_output(request)?;
                Ok(ResponseHandler::Ok)
            }
            Request::UnregisterOutput { id } => {
                self.pipeline.unregister_output(&id)?;
                Ok(ResponseHandler::Ok)
            }
            Request::Start => {
                self.pipeline.start();
                Ok(ResponseHandler::Ok)
            }
            Request::UpdateScene(scene_spec) => {
                self.pipeline.update_scene(scene_spec)?;
                Ok(ResponseHandler::Ok)
            }
            Request::RegisterTransformation {
                key,
                transform: spec,
            } => {
                self.pipeline.register_transformation(key, spec)?;
                Ok(ResponseHandler::Ok)
            }
            Request::Query(query) => self.handle_query(query),
        }
    }

    fn handle_query(&self, query: QueryRequest) -> Result<ResponseHandler> {
        match query {
            QueryRequest::WaitForNextFrame { input_id } => {
                let (sender, receiver) = bounded(1);
                self.pipeline.queue().subscribe_input_listener(
                    input_id,
                    Box::new(move || {
                        sender.send(Ok(Response::Ok {})).unwrap();
                    }),
                );
                Ok(ResponseHandler::DeferredResponse(receiver))
            }
            QueryRequest::Scene => Ok(ResponseHandler::Response(Response::Scene(
                self.pipeline.renderer().scene_spec(),
            ))),
            QueryRequest::Inputs => {
                let inputs = self
                    .pipeline
                    .inputs()
                    .map(|(id, node)| InputInfo {
                        id: id.clone(),
                        port: node.port,
                    })
                    .collect();
                Ok(ResponseHandler::Response(Response::Inputs { inputs }))
            }
            QueryRequest::Outputs => {
                let outputs = self.pipeline.with_outputs(|iter| {
                    iter.map(|(id, node)| OutputInfo {
                        id: id.clone(),
                        port: node.port,
                        ip: node.ip.clone(),
                    })
                    .collect()
                });
                Ok(ResponseHandler::Response(Response::Outputs { outputs }))
            }
        }
    }

    fn register_output(&mut self, request: RegisterOutputRequest) -> Result<()> {
        let RegisterOutputRequest {
            id,
            port,
            resolution,
            encoder_settings,
            ip,
        } = request;

        self.pipeline.with_outputs(|mut iter| {
            if let Some((node_id, _)) = iter.find(|(_, output)| output.port == port && output.ip == ip) {
                return Err(anyhow!(
                    "Failed to register output with id \"{id}\". Combination of port {port} and IP {ip} is already used by node \"{node_id}\""
                ));
            };
            Ok(())
        })?;

        self.pipeline.register_output(
            id,
            rtp_sender::Options {
                port,
                ip,
                resolution,
                encoder_settings,
            },
        )?;

        Ok(())
    }

    fn register_input(&mut self, request: RegisterInputRequest) -> Result<()> {
        let RegisterInputRequest { id, port } = request;

        if let Some((node_id, _)) = self.pipeline.inputs().find(|(_, input)| input.port == port) {
            return Err(anyhow!(
                "Failed to register input with id \"{id}\". Port {port} is already used by node \"{node_id}\""
            ));
        }

        self.pipeline
            .register_input(id.clone(), rtp_receiver::Options { port, input_id: id })?;

        Ok(())
    }
}
