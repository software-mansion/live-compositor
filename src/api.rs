use std::sync::Arc;

use anyhow::{anyhow, Result};
use compositor_common::{
    scene::{InputId, OutputId, Resolution, SceneSpec},
    transformation::{TransformationRegistryKey, TransformationSpec},
};
use compositor_pipeline::pipeline;
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
    pub ip: String,
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
    Start,
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

    pub fn handle_request(&mut self, request: Request) -> Result<()> {
        match request {
            Request::Init(_) => Err(anyhow!("Video compositor is already initialized.")),
            Request::RegisterInput(request) => self.register_input(request),
            Request::UnregisterInput { id } => Ok(self.pipeline.unregister_input(&id)?),
            Request::RegisterOutput(request) => self.register_output(request),
            Request::UnregisterOutput { id } => Ok(self.pipeline.unregister_output(&id)?),
            Request::Start => {
                self.pipeline.start();
                Ok(())
            }
            Request::UpdateScene(scene_spec) => Ok(self.pipeline.update_scene(scene_spec)?),
            Request::RegisterTransformation {
                key,
                transform: spec,
            } => Ok(self.pipeline.register_transformation(key, spec)?),
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

        //if let Some((node_id, _)) = state.inputs.iter().find(|(_, input)| input.port == port) {
        //    return Err(anyhow!(
        //        "Failed to register input with {id}. Port {port} is already use by node {node_id}"
        //    ));
        //}

        self.pipeline
            .register_input(id.clone(), rtp_receiver::Options { port, input_id: id })?;

        Ok(())
    }
}
