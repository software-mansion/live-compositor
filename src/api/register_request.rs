use compositor_pipeline::pipeline::{self, Port};

use crate::{
    error::ApiError,
    types::{RegisterOutputRequest, RegisterRequest},
};

use super::{Api, ResponseHandler};

pub fn handle_register_request(
    api: &mut Api,
    request: RegisterRequest,
) -> Result<Option<ResponseHandler>, ApiError> {
    match request {
        RegisterRequest::InputStream(input_stream) => {
            let register_options = input_stream.try_into()?;
            let Port(port) = api.pipeline.register_input(register_options)?;
            Ok(Some(ResponseHandler::Response(
                super::Response::RegisteredPort(port),
            )))
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

    api.pipeline
        .register_output(output_id.into(), request.clone().into(), request.into())?;

    Ok(())
}
