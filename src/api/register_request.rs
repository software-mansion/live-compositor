use compositor_pipeline::{
    error::{InputInitError, RegisterInputError},
    pipeline::{
        self,
        decoder::{AudioDecoderOptions, DecoderOptions, OpusDecoderOptions, VideoDecoderOptions},
        input::{
            rtp::{AudioStream, RtpReceiverError, RtpReceiverOptions, RtpStream, VideoStream},
            InputOptions,
        },
    },
};
use log::trace;

use crate::{
    api::Response,
    error::{ApiError, PORT_ALREADY_IN_USE_ERROR_CODE},
    rtp_sender,
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
    let RegisterInputRequest {
        input_id: id,
        port,
        video,
        audio,
    } = request;
    let port: Port = port.try_into()?;

    let (rtp_stream, decoder_opts) = input_options(video, audio);

    match port {
        Port::Range((start, end)) => {
            for port in start..=end {
                trace!("[input {id}] checking port {port}");

                if api
                    .pipeline
                    .inputs()
                    // flat_map so that you can skip other inputs in the future by doing => None on them
                    .flat_map(|(_, input)| match input.input {
                        pipeline::input::Input::Rtp(ref rtp) => Some(rtp),
                    })
                    .any(|input| input.port == port || input.port + 1 == port)
                {
                    trace!("[input {id}] port {port} is already used by another input",);
                    continue;
                }

                let input_opts = pipeline::input::InputOptions::Rtp(RtpReceiverOptions {
                    port,
                    input_id: id.clone().into(),
                    stream: rtp_stream.clone(),
                });

                let result = api.pipeline.register_input(
                    id.clone().into(),
                    input_opts,
                    decoder_opts.clone(),
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
            if let Some((node_id, _)) = api
                .pipeline
                .inputs()
                // flat_map so that you can skip other inputs in the future by doing => None on them
                .flat_map(|(id, input)| match input.input {
                    pipeline::input::Input::Rtp(ref rtp) => Some((id, rtp)),
                })
                .find(|(_, input)| input.port == port)
            {
                return Err(ApiError::new(
                    PORT_ALREADY_IN_USE_ERROR_CODE,
                    format!("Failed to register input stream \"{id}\". Port {port} is already used by node \"{node_id}\""),
                    tiny_http::StatusCode(400)
                ));
            }

            let input_opts = InputOptions::Rtp(RtpReceiverOptions {
                port,
                input_id: id.clone().into(),
                stream: rtp_stream,
            });

            let result = api
                .pipeline
                .register_input(id.clone().into(), input_opts, decoder_opts);

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
    let Err(RegisterInputError::InputError(ref id, err)) = register_input_error else {
        return Ok(());
    };

    let InputInitError::Rtp(RtpReceiverError::SocketBind(ref err)) = err else {
        return Ok(());
    };

    match err.kind() {
        std::io::ErrorKind::AddrInUse =>
            Err(ApiError::new(
                PORT_ALREADY_IN_USE_ERROR_CODE,
                format!("Failed to register input stream \"{id}\". Port {port} is already in use or not available."),
                tiny_http::StatusCode(400)
            )),
        _ => Ok(())
    }
}

fn input_options(
    video: Option<crate::types::Video>,
    audio: Option<crate::types::Audio>,
) -> (RtpStream, DecoderOptions) {
    let video_decoder_opts = video.clone().map(|video| VideoDecoderOptions {
        codec: video.codec.into(),
    });
    let audio_decoder_opts = audio.clone().map(|audio| match audio.codec {
        crate::types::AudioCodec::Opus => AudioDecoderOptions::Opus(OpusDecoderOptions {
            sample_rate: audio.sample_rate,
            channels: audio.channels.into(),
            forward_error_correction: audio.forward_error_correction.unwrap_or(false),
        }),
    });
    let decoder_opts = if let (None, None) = (&video_decoder_opts, &audio_decoder_opts) {
        DecoderOptions {
            video: Some(VideoDecoderOptions {
                codec: crate::types::VideoCodec::default().into(),
            }),
            audio: None,
        }
    } else {
        DecoderOptions {
            video: video_decoder_opts,
            audio: audio_decoder_opts,
        }
    };
    let rtp_stream = RtpStream {
        video: video.map(|video| VideoStream {
            codec: video.codec.into(),
            payload_type: video.rtp_payload_type.unwrap_or(96),
        }),
        audio: audio.map(|audio| AudioStream {
            codec: audio.codec.into(),
            payload_type: audio.rtp_payload_type.unwrap_or(97),
        }),
    };

    (rtp_stream, decoder_opts)
}

impl From<crate::types::VideoCodec> for pipeline::structs::VideoCodec {
    fn from(value: crate::types::VideoCodec) -> Self {
        match value {
            crate::types::VideoCodec::H264 => pipeline::structs::VideoCodec::H264,
        }
    }
}

impl From<crate::types::AudioCodec> for pipeline::structs::AudioCodec {
    fn from(value: crate::types::AudioCodec) -> Self {
        match value {
            crate::types::AudioCodec::Opus => pipeline::structs::AudioCodec::Opus,
        }
    }
}

impl From<crate::types::AudioChannels> for pipeline::structs::AudioChannels {
    fn from(value: crate::types::AudioChannels) -> Self {
        match value {
            crate::types::AudioChannels::Mono => pipeline::structs::AudioChannels::Mono,
            crate::types::AudioChannels::Stereo => pipeline::structs::AudioChannels::Stereo,
        }
    }
}
