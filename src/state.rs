use anyhow::Result;
use compositor_common::scene::Resolution;
use compositor_pipeline::map::SyncHashMap;
use std::sync::Arc;

use crate::{rtp_receiver::RtpReceiver, rtp_sender::RtpSender};

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
    pub inputs: SyncHashMap<u16, Input>,
    pub outputs: SyncHashMap<u16, Output>,
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

    #[allow(dead_code)]
    pub fn register_output(&self, port: u16, resolution: Resolution) -> Result<()> {
        // TODO: add validation if output already relisted
        let sender = Arc::new(RtpSender::new(port, resolution));
        self.pipeline.add_output(port.into(), sender.clone());
        self.outputs.insert(
            port,
            Output {
                port,
                rtp_sender: sender,
            },
        );
        Ok(())
    }

    #[allow(dead_code)]
    pub fn register_input(&self, port: u16) -> Result<()> {
        // TODO: add validation if input already relisted
        self.pipeline.add_input(port.into());
        self.inputs.insert(
            port,
            Input {
                port,
                rtp_receiver: RtpReceiver::new(self.pipeline.clone(), port),
            },
        );
        Ok(())
    }
}
