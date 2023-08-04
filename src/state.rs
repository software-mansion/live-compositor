use anyhow::Result;
use compositor_common::{
    scene::{InputId, OutputId, SceneSpec},
    transformation::{TransformationRegistryKey, TransformationSpec},
};
use compositor_pipeline::map::SyncHashMap;

use std::sync::Arc;

use crate::{
    http::{RegisterInputRequest, RegisterOutputRequest},
    rtp_receiver::RtpReceiver,
    rtp_sender::{self, RtpSender},
};

pub type Pipeline = compositor_pipeline::Pipeline<RtpSender>;

#[allow(dead_code)]
pub struct Input {
    port: u16,
    rtp_receiver: RtpReceiver,
}

#[allow(dead_code)]
pub struct Output {
    port: u16,
    rtp_sender: Arc<RtpSender>,
}

#[allow(dead_code)]
pub struct InitConfig {
    // some init data
}

pub struct State {
    pub inputs: SyncHashMap<InputId, Input>,
    pub outputs: SyncHashMap<OutputId, Output>,
    pub pipeline: Arc<Pipeline>,
}

impl State {
    pub fn new(pipeline: Arc<Pipeline>) -> State {
        State {
            inputs: SyncHashMap::new(),
            outputs: SyncHashMap::new(),
            pipeline,
        }
    }

    pub fn register_output(&self, request: RegisterOutputRequest) -> Result<()> {
        let RegisterOutputRequest {
            id,
            port,
            resolution,
            encoder_settings,
            ip,
        } = request;
        // TODO: add validation if output already relisted
        let sender = Arc::new(RtpSender::new(rtp_sender::Options {
            port,
            ip,
            resolution,
            encoder_settings,
        }));
        self.pipeline.add_output(id.clone(), sender.clone());
        self.outputs.insert(
            id,
            Output {
                port,
                rtp_sender: sender,
            },
        );
        Ok(())
    }

    pub fn register_input(&self, request: RegisterInputRequest) -> Result<()> {
        let RegisterInputRequest { id, port } = request;
        // TODO: add validation if input already relisted
        self.pipeline.add_input(id.clone());
        self.inputs.insert(
            id.clone(),
            Input {
                port,
                rtp_receiver: RtpReceiver::new(self.pipeline.clone(), port, id),
            },
        );
        Ok(())
    }

    pub fn update_scene(&self, scene: SceneSpec) -> Result<()> {
        scene.validate(
            &self.inputs.keys().into_iter().map(|i| i.0).collect(),
            &self.outputs.keys().into_iter().map(|i| i.0).collect(),
        )?;
        self.pipeline.update_scene(scene)?;
        Ok(())
    }

    pub fn register_transformation(
        &self,
        key: TransformationRegistryKey,
        transformation_spec: TransformationSpec,
    ) -> Result<()> {
        self.pipeline
            .register_transformation(key, transformation_spec)?;
        Ok(())
    }
}
