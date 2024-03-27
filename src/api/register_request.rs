use compositor_pipeline::pipeline::{Port, RegisterInputOptions};
use compositor_render::InputId;

use crate::{error::ApiError, types::RegisterRequest};

use super::{Api, Pipeline, Response};

fn handle_register_input(
    api: &Api,
    input_id: InputId,
    register_options: RegisterInputOptions,
) -> Result<Response, ApiError> {
    match Pipeline::register_input(&api.pipeline, input_id, register_options)? {
        Some(Port(port)) => Ok(Response::RegisteredPort { port }),
        None => Ok(Response::Ok {}),
    }
}

pub async fn handle_register_request(
    api: &Api,
    request: RegisterRequest,
) -> Result<Response, ApiError> {
    let api = api.clone();
    let response = tokio::task::spawn_blocking(move || match request {
        RegisterRequest::RtpInputStream(rtp) => {
            let (input_id, register_options) = rtp.try_into()?;
            handle_register_input(&api, input_id, register_options)
        }
        RegisterRequest::Mp4(mp4) => {
            let (input_id, register_options) = mp4.try_into()?;
            handle_register_input(&api, input_id, register_options)
        }
        RegisterRequest::OutputStream(output_stream) => {
            match api.pipeline().register_output(output_stream.try_into()?)? {
                Some(Port(port)) => Ok(Response::RegisteredPort { port }),
                None => Ok(Response::Ok {}),
            }
        }
        RegisterRequest::Shader(spec) => {
            let spec = spec.try_into()?;
            Pipeline::register_renderer(&api.pipeline, spec)?;
            Ok(Response::Ok {})
        }
        RegisterRequest::WebRenderer(spec) => {
            let spec = spec.try_into()?;
            Pipeline::register_renderer(&api.pipeline, spec)?;
            Ok(Response::Ok {})
        }
        RegisterRequest::Image(spec) => {
            let spec = spec.try_into()?;
            Pipeline::register_renderer(&api.pipeline, spec)?;
            Ok(Response::Ok {})
        }
    });

    // `unwrap()` panics only when the task panicked or `response.abort()` was called
    response.await.unwrap()
}
