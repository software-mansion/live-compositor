use compositor_pipeline::pipeline::{Port, RegisterInputOptions};
use compositor_render::InputId;

use crate::{error::ApiError, types::RegisterRequest};

use super::{Api, ResponseHandler};

fn handle_register_input(
    api: &mut Api,
    input_id: InputId,
    register_options: RegisterInputOptions,
) -> Result<ResponseHandler, ApiError> {
    match api.pipeline().register_input(input_id, register_options)? {
        Some(Port(port)) => Ok(ResponseHandler::Response(super::Response::RegisteredPort {
            port,
        })),

        None => Ok(ResponseHandler::Ok),
    }
}

pub fn handle_register_request(
    api: &mut Api,
    request: RegisterRequest,
) -> Result<ResponseHandler, ApiError> {
    match request {
        RegisterRequest::RtpInputStream(rtp) => {
            let (input_id, register_options) = rtp.try_into()?;
            handle_register_input(api, input_id, register_options)
        }
        RegisterRequest::Mp4(mp4) => {
            let (input_id, register_options) = mp4.try_into()?;
            handle_register_input(api, input_id, register_options)
        }
        RegisterRequest::OutputStream(output_options) => {
            api.pipeline().register_output(output_options.try_into()?)?;
            Ok(ResponseHandler::Ok)
        }
        RegisterRequest::Shader(spec) => {
            let spec = spec.try_into()?;
            api.pipeline().register_renderer(spec)?;
            Ok(ResponseHandler::Ok)
        }
        RegisterRequest::WebRenderer(spec) => {
            let spec = spec.try_into()?;
            api.pipeline().register_renderer(spec)?;
            Ok(ResponseHandler::Ok)
        }
        RegisterRequest::Image(spec) => {
            let spec = spec.try_into()?;
            api.pipeline().register_renderer(spec)?;
            Ok(ResponseHandler::Ok)
        }
    }
}
