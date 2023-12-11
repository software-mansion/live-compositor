use compositor_pipeline::{
    error::{InputInitError, RegisterInputError},
    pipeline,
};
use log::trace;

use crate::{
    api::Response,
    error::{ApiError, PORT_ALREADY_IN_USE_ERROR_CODE},
    rtp_receiver, rtp_sender,
    types::{RegisterInputRequest, RegisterOutputRequest, RegisterRequest},
};

use super::{Api, Port, ResponseHandler};

pub fn handle_register_request(
    api: &mut Api,
    request: RegisterRequest,
) -> Result<Option<ResponseHandler>, ApiError> {
    match request {
        RegisterRequest::InputStream(input_stream) => register_input(api, input_stream).map(Some),
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
        resolution,
        encoder_settings,
        ip,
    } = request;

    api.pipeline.with_outputs(|mut iter| {
        if let Some((node_id, _)) = iter.find(|(_, output)| output.port == port && output.ip == ip) {
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
        pipeline::OutputOptions {
            resolution: resolution.into(),
            encoder_settings: encoder_settings.into(),
            receiver_options: rtp_sender::Options { port, ip },
        },
    )?;

    Ok(())
}

fn register_input(
    api: &mut Api,
    request: RegisterInputRequest,
) -> Result<ResponseHandler, ApiError> {
    let RegisterInputRequest { input_id: id, port } = request;
    let port: Port = port.try_into()?;

    match port {
        Port::Range((start, end)) => {
            for port in start..=end {
                trace!("[input {id}] checking port {port}");

                if api
                    .pipeline
                    .inputs()
                    .any(|(_, input)| input.port == port || input.port + 1 == port)
                {
                    trace!("[input {id}] port {port} is already used by another input",);
                    continue;
                }

                let result = api.pipeline.register_input(
                    id.clone().into(),
                    rtp_receiver::Options {
                        port,
                        input_id: id.clone(),
                    },
                );

                if check_port_not_available(&result, port).is_err() {
                    trace!(
                        "[input {id}] FFmpeg reported port registration failure for port {port}",
                    );
                    continue;
                }

                return match result {
                    Ok(_) => {
                        trace!("[input {id}] port registration succeeded for port {port}");
                        Ok(ResponseHandler::Response(Response::RegisteredPort(port)))
                    }
                    Err(e) => Err(e.into()),
                };
            }

            Err(ApiError::new(
                PORT_ALREADY_IN_USE_ERROR_CODE,
                format!("Failed to register input stream \"{id}\". Ports {start}..{end} are already used or not available."),
                tiny_http::StatusCode(400)
            ))
        }

        Port::Exact(port) => {
            if let Some((node_id, _)) = api.pipeline.inputs().find(|(_, input)| input.port == port)
            {
                return Err(ApiError::new(
                    PORT_ALREADY_IN_USE_ERROR_CODE,
                    format!("Failed to register input stream \"{id}\". Port {port} is already used by node \"{node_id}\""),
                    tiny_http::StatusCode(400)
                ));
            }

            let result = api.pipeline.register_input(
                id.clone().into(),
                rtp_receiver::Options { port, input_id: id },
            );

            check_port_not_available(&result, port)?;

            result?;

            Ok(ResponseHandler::Response(Response::RegisteredPort(port)))
        }
    }
}

/// Returns Ok(()) if there isn't an error or the error is not a port already in use error.
/// Returns Err(ApiError) if the error is a port already in use error.
fn check_port_not_available<T>(
    register_input_error: &Result<T, RegisterInputError>,
    port: u16,
) -> Result<(), ApiError> {
    if let Err(RegisterInputError::DecoderError(ref id, InputInitError::InputError(ref err))) =
        register_input_error
    {
        if let Some(err) = err.0.downcast_ref::<std::io::Error>() {
            match err.kind() {
                std::io::ErrorKind::AddrInUse =>
                    Err(ApiError::new(
                        PORT_ALREADY_IN_USE_ERROR_CODE,
                        format!("Failed to register input stream \"{id}\". Port {port} is already in use or not available."),
                        tiny_http::StatusCode(400)
                    )),
                _ => Ok(())
            }
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}
