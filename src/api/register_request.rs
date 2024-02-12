use compositor_pipeline::pipeline::{self, Port, RegisterInputOptions};
use compositor_render::InputId;

use crate::{
    error::ApiError,
    types::{RegisterOutputRequest, RegisterRequest},
};

use super::{Api, ResponseHandler};

fn handle_register_input(
    api: &mut Api,
    input_id: InputId,
    register_options: RegisterInputOptions,
) -> Result<Option<ResponseHandler>, ApiError> {
    match api.pipeline.register_input(input_id, register_options)? {
        Some(Port(port)) => Ok(Some(ResponseHandler::Response(
            super::Response::RegisteredPort { port },
        ))),

        None => Ok(Some(ResponseHandler::Ok)),
    }
}

pub fn handle_register_request(
    api: &mut Api,
    request: RegisterRequest,
) -> Result<Option<ResponseHandler>, ApiError> {
    match request {
        RegisterRequest::RtpInputStream(rtp) => {
            let (input_id, register_options) = rtp.try_into()?;
            handle_register_input(api, input_id, register_options)
        }
        RegisterRequest::Mp4(mp4) => {
            let (input_id, register_options) = mp4.try_into()?;
            handle_register_input(api, input_id, register_options)
        }
        RegisterRequest::OutputStream(output_stream) => {
            register_output(api, output_stream)?;
            Ok(None)
        }
        RegisterRequest::Shader(spec) => {
            let spec = spec.try_into()?;
            api.pipeline.register_renderer(spec)?;
            Ok(None)
        }
        RegisterRequest::WebRenderer(spec) => {
            let spec = spec.try_into()?;
            api.pipeline.register_renderer(spec)?;
            Ok(None)
        }
        RegisterRequest::Image(spec) => {
            let spec = spec.try_into()?;
            api.pipeline.register_renderer(spec)?;
            Ok(None)
        }
    }
}

fn register_output(api: &mut Api, request: RegisterOutputRequest) -> Result<(), ApiError> {
    let ip = request.ip.clone();
    let port = request.port;
    api.pipeline.with_outputs(|mut iter| {
        if let Some((node_id, _)) = iter.find(|(_, output)| match &output.output {
            pipeline::output::Output::Rtp(rtp) => rtp.port == port && rtp.ip == ip,
        }) {
            let output_id = request.output_id.clone();
            return Err(ApiError::new(
                "PORT_AND_IP_ALREADY_IN_USE",
                format!("Failed to register output stream \"{output_id}\". Combination of port {port} and IP {ip} is already used by node \"{node_id}\""),
                tiny_http::StatusCode(400)
            ));
        };
        Ok(())
    })?;
    api.pipeline.register_output(request.try_into()?)?;

    Ok(())
}
