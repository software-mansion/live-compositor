use compositor_pipeline::pipeline::{self, Port, RegisterInputOptions};

use crate::{
    error::ApiError,
    types::{RegisterOutputRequest, RegisterRequest},
};

use super::{Api, ResponseHandler};

fn handle_register_input(
    api: &mut Api,
    register_options: RegisterInputOptions,
) -> Result<Option<ResponseHandler>, ApiError> {
    match api.pipeline.register_input(register_options)? {
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
            let register_options = rtp.try_into()?;
            handle_register_input(api, register_options)
        }
        RegisterRequest::Mp4(mp4) => {
            let register_options = mp4.try_into()?;
            handle_register_input(api, register_options)
        }
        RegisterRequest::OutputStream(output_stream) => {
            register_output(api, output_stream).map(|_| None)
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
    let RegisterOutputRequest {
        output_id,
        port,
        ip,
        ..
    } = request.clone();

    api.pipeline.with_outputs(|mut iter| {
        if let Some((node_id, _)) = iter.find(|(_, output)| match &output.output {
            pipeline::output::Output::Rtp(rtp) => rtp.port == port && rtp.ip == ip,
        }) {
            return Err(ApiError::new(
                "PORT_AND_IP_ALREADY_IN_USE",
                format!("Failed to register output stream \"{output_id}\". Combination of port {port} and IP {ip} is already used by node \"{node_id}\""),
                tiny_http::StatusCode(400)
            ));
        };
        Ok(())
    })?;

    api.pipeline.register_output(
        output_id.into(),
        request.clone().into(),
        request.clone().into(),
        request.initial_scene.try_into()?,
    )?;

    Ok(())
}
