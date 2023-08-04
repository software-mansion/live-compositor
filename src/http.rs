use anyhow::{anyhow, Result};
use compositor_common::{
    scene::{InputId, OutputId, Resolution, SceneSpec},
    transformation::{TransformationRegistryKey, TransformationSpec},
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{io::Cursor, net::SocketAddr, sync::Arc, thread};
use tiny_http::{Response, StatusCode};

use crate::rtp_sender::EncoderSettings;

use super::state::State;

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
enum Request {
    RegisterInput(RegisterInputRequest),
    RegisterOutput(RegisterOutputRequest),
    UpdateScene(SceneSpec),
    RegisterTransformation {
        key: TransformationRegistryKey,
        transform: TransformationSpec,
    },
    Init,
    Start,
}

pub struct Server {
    server: tiny_http::Server,
    state: Arc<State>,
}

impl Server {
    pub fn new(port: u16, state: Arc<State>) -> Self {
        Self {
            server: tiny_http::Server::http(SocketAddr::from(([0, 0, 0, 0], port))).unwrap(),
            state,
        }
    }

    pub fn start(self) {
        info!("Listening on port {}", self.server.server_addr());
        for mut raw_request in self.server.incoming_requests() {
            let result = self.handle_request_before_init(&mut raw_request);
            let should_abort = result.is_ok();
            Server::send_response(raw_request, result);
            if should_abort {
                break;
            }
        }
        thread::spawn(move || {
            for mut raw_request in self.server.incoming_requests() {
                let result = self.handle_request_after_init(&mut raw_request);
                Server::send_response(raw_request, result);
            }
        });
    }

    fn handle_request_after_init(&self, raw_request: &mut tiny_http::Request) -> Result<()> {
        let request = Server::parse_request(raw_request)?;
        match request {
            Request::Init => Err(anyhow!("Video composer is already configured.")),
            Request::RegisterInput(request) => self.state.register_input(request),
            Request::RegisterOutput(request) => self.state.register_output(request),
            Request::Start => {
                self.state.pipeline.start();
                Ok(())
            }
            Request::UpdateScene(scene_spec) => self.state.update_scene(scene_spec),
            Request::RegisterTransformation {
                key,
                transform: spec,
            } => self.state.register_transformation(key, spec),
        }
    }

    fn handle_request_before_init(&self, raw_request: &mut tiny_http::Request) -> Result<()> {
        let request = Server::parse_request(raw_request)?;
        match request {
            Request::Init => Ok(()),
            _ => Err(anyhow!(
                "No requests are supported before init request is completed."
            )),
        }
    }

    fn send_response(raw_request: tiny_http::Request, response: Result<()>) {
        let response_result = match response {
            Ok(_) => raw_request.respond(Response::new(
                StatusCode(200),
                vec![],
                Cursor::new("{}".as_bytes().to_vec()),
                None,
                None,
            )),
            Err(err) => raw_request.respond(Response::new(
                StatusCode(500),
                vec![],
                Cursor::new(format!("{}", err).into_bytes()),
                None,
                None,
            )),
        };
        if let Err(err) = response_result {
            error!("Failed to send response {}.", err);
        }
    }

    fn parse_request(request: &mut tiny_http::Request) -> Result<Request> {
        Ok(serde_json::from_reader::<_, Request>(request.as_reader())?)
    }
}
