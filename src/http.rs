use anyhow::{anyhow, Result};
use compositor_common::SpecValidationError;
use compositor_pipeline::pipeline;
use compositor_render::registry;
use log::{error, info};

use serde_json::json;
use std::{error::Error, io::Cursor, net::SocketAddr, thread};
use tiny_http::{Header, Response, StatusCode};

use crate::api::{Api, Request};

pub struct Server {
    server: tiny_http::Server,
    content_type_json: Header,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Self {
            server: tiny_http::Server::http(SocketAddr::from(([0, 0, 0, 0], port))).unwrap(),
            content_type_json: Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
                .unwrap(),
        }
    }

    pub fn start(self) {
        info!("Listening on port {}", self.server.server_addr());
        let mut api = self.handle_init();
        thread::spawn(move || {
            for mut raw_request in self.server.incoming_requests() {
                let result = Self::handle_request_after_init(&mut api, &mut raw_request);
                self.send_response(raw_request, result);
            }
        });
    }

    fn handle_init(&self) -> Api {
        for mut raw_request in self.server.incoming_requests() {
            let result = self
                .handle_request_before_init(&mut raw_request)
                .and_then(Api::new);
            match result {
                Ok(new_api) => {
                    self.send_response(raw_request, Ok(()));
                    return new_api;
                }
                Err(err) => {
                    self.send_response(raw_request, Err(err));
                }
            }
        }
        panic!("Server shutdown unexpectedly.")
    }

    fn handle_request_after_init(
        api: &mut Api,
        raw_request: &mut tiny_http::Request,
    ) -> Result<()> {
        let request = Server::parse_request(raw_request)?;
        api.handle_request(request)
    }

    fn handle_request_before_init(
        &self,
        raw_request: &mut tiny_http::Request,
    ) -> Result<pipeline::Options> {
        let request = Server::parse_request(raw_request)?;
        match request {
            Request::Init(opts) => Ok(opts),
            _ => Err(anyhow!(
                "No requests are supported before init request is completed."
            )),
        }
    }

    fn send_response(&self, raw_request: tiny_http::Request, response: Result<()>) {
        let response_result = match response {
            Ok(_) => {
                let body = "{}".as_bytes().to_vec();
                raw_request
                    .respond(Response::new(
                        StatusCode(200),
                        vec![self.content_type_json.clone()],
                        Cursor::new(&body),
                        Some(body.len()),
                        None,
                    ))
                    .map_err(Into::into)
            }
            Err(err) => self.handle_error(raw_request, err.as_ref()),
        };
        if let Err(err) = response_result {
            error!("Failed to send response {}.", err);
        }
    }

    fn handle_error(
        &self,
        raw_request: tiny_http::Request,
        err: &(dyn Error + 'static),
    ) -> Result<()> {
        let reason: Vec<String> = Sources(Some(err)).map(|e| format!("{e}")).collect();
        let body = serde_json::to_string(&json!({
            "msg": reason[0],
            "reason": reason,
        }))?;
        let status_code = match is_user_error(err) {
            true => 400,
            false => 500,
        };
        raw_request.respond(Response::new(
            StatusCode(status_code),
            vec![self.content_type_json.clone()],
            Cursor::new(&body),
            Some(body.len()),
            None,
        ))?;
        Ok(())
    }

    fn parse_request(request: &mut tiny_http::Request) -> Result<Request> {
        Ok(serde_json::from_reader::<_, Request>(request.as_reader())?)
    }
}

fn is_user_error(err: &(dyn Error + 'static)) -> bool {
    let opt = Some(err);
    let mut sources = Sources(opt);
    sources.any(|source| {
        source.is::<SpecValidationError>()
            || source.is::<registry::RegisterError>()
            || source.is::<registry::GetError>()
        // TODO: add more errors here
    })
}

/// Replace with err.sources() when the feature stabilizes
struct Sources<'a>(Option<&'a (dyn Error + 'static)>);

impl<'a> Iterator for Sources<'a> {
    type Item = &'a (dyn Error + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            Some(err) => {
                self.0 = err.source();
                Some(err)
            }
            None => None,
        }
    }
}
