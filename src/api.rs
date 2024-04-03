use std::sync::{Arc, Mutex, MutexGuard};

use compositor_pipeline::pipeline::{self};
use compositor_render::{error::InitPipelineError, EventLoop};

use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    types::{InputId, OutputId},
};

pub type Pipeline = compositor_pipeline::Pipeline;

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

    // pub async fn handle_request(&self, request: Request) -> Result<Response, ApiError> {
    //     match request {
    //         Request::Register(register_request) => {
    //             register_request::handle_register_request(self, register_request).await
    //         }
    //         Request::Unregister(unregister_request) => {
    //             unregister_request::handle_unregister_request(self, unregister_request)
    //         }
    //         Request::Start => {
    //             Pipeline::start(&self.pipeline);
    //             Ok(Response::Ok {})
    //         }
    //         Request::UpdateOutput(update) => self.handle_scene_update(update),
    //     }
    // }

    pub(crate) fn pipeline(&self) -> MutexGuard<'_, Pipeline> {
        self.pipeline.lock().unwrap()
    }
}
