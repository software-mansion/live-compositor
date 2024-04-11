use std::time::Duration;

use bytes::{BufMut, Bytes, BytesMut};
use compositor_pipeline::{
    pipeline::{
        self, decoder,
        input::{self, rtp::InputAudioStream},
    },
    queue,
};

use super::register_input::*;
use super::util::*;

/// [RFC 3640, section 4.1. MIME Type Registration (`config` subsection)](https://datatracker.ietf.org/doc/html/rfc3640#section-4.1)
fn parse_asc(asc: Option<&str>) -> Result<Option<Bytes>, TypeError> {
    let Some(asc) = asc else {
        return Ok(None);
    };

    const EMPTY_ASC: &str = "The AudioSpecificConfig field is empty. Either omit this field or provide a non-empty ASC.";
    if asc.is_empty() {
        return Err(TypeError::new(EMPTY_ASC));
    }

    let mut output = BytesMut::new();

    const BAD_ASC_FORMAT: &str = "Not all of the provided AudioSpecificConfig is hex digits.";
    if !asc.chars().all(|c| char::is_ascii_hexdigit(&c)) {
        return Err(TypeError::new(BAD_ASC_FORMAT));
    }

    for a in asc.as_bytes().chunks(2) {
        let fallback = &[a[0], 0];
        let val = if a.len() == 2 { a } else { fallback };
        // the value is already checked for being ascii
        let val = String::from_utf8_lossy(val);

        const BYTE_PARSE_ERROR: &str = "An error occurred while parsing a byte of the ASC";
        let parsed = match u8::from_str_radix(&val, 16) {
            Ok(res) => res,
            // This can't happen, but just to be safe
            Err(_) => return Err(TypeError::new(BYTE_PARSE_ERROR)),
        };
        output.put_u8(parsed);
    }

    Ok(Some(output.freeze()))
}

fn convert_audio(
    audio: Option<&InputRtpAudioOptions>,
) -> Result<Option<InputAudioStream>, TypeError> {
    let Some(audio) = audio else {
        return Ok(None);
    };

    match audio {
        InputRtpAudioOptions::Opus {
            forward_error_correction,
        } => {
            let forward_error_correction = forward_error_correction.unwrap_or(false);
            Ok(Some(input::rtp::InputAudioStream {
                options: decoder::AudioDecoderOptions::Opus(decoder::OpusDecoderOptions {
                    forward_error_correction,
                }),
            }))
        }
        InputRtpAudioOptions::Aac {
            audio_specific_config,
            rtp_mode,
        } => {
            let depayloader_mode = match rtp_mode {
                Some(AacRtpMode::LowBitrate) => Some(decoder::AacDepayloaderMode::LowBitrate),
                Some(AacRtpMode::HighBitrate) | None => {
                    Some(decoder::AacDepayloaderMode::HighBitrate)
                }
            };

            let asc = parse_asc(audio_specific_config.as_deref())?;

            Ok(Some(input::rtp::InputAudioStream {
                options: decoder::AudioDecoderOptions::Aac(decoder::AacDecoderOptions {
                    depayloader_mode,
                    asc,
                }),
            }))
        }
    }
}

impl TryFrom<RtpInputStream> for pipeline::RegisterInputOptions {
    type Error = TypeError;

    fn try_from(value: RtpInputStream) -> Result<Self, Self::Error> {
        let RtpInputStream {
            port,
            video,
            audio,
            required,
            offset_ms,
            transport_protocol,
        } = value;

        const NO_VIDEO_AUDIO_SPEC: &str =
            "At least one of `video` and `audio` has to be specified in `register_input` request.";

        if video.is_none() && audio.is_none() {
            return Err(TypeError::new(NO_VIDEO_AUDIO_SPEC));
        }

        let audio = convert_audio(audio.as_ref())?;

        let rtp_stream = input::rtp::RtpStream {
            video: video.as_ref().map(|video| input::rtp::InputVideoStream {
                options: match video {
                    InputRtpVideoOptions::FfmepgH264 => decoder::VideoDecoderOptions {
                        codec: pipeline::VideoCodec::H264,
                    },
                },
            }),
            audio,
        };

        let input_options = input::InputOptions::Rtp(input::rtp::RtpReceiverOptions {
            port: port.try_into()?,
            stream: rtp_stream,
            transport_protocol: transport_protocol.unwrap_or(TransportProtocol::Udp).into(),
        });

        let queue_options = queue::InputOptions {
            required: required.unwrap_or(false),
            offset: offset_ms.map(|offset_ms| Duration::from_secs_f64(offset_ms / 1000.0)),
        };

        Ok(pipeline::RegisterInputOptions {
            input_options,
            queue_options,
        })
    }
}

impl TryFrom<Mp4> for pipeline::RegisterInputOptions {
    type Error = TypeError;

    fn try_from(value: Mp4) -> Result<Self, Self::Error> {
        let Mp4 {
            url,
            path,
            required,
            offset_ms,
        } = value;

        const BAD_URL_PATH_SPEC: &str =
            "Exactly one of `url` or `path` has to be specified in a register request for an mp4 input.";

        let source = match (url, path) {
            (Some(_), Some(_)) | (None, None) => {
                return Err(TypeError::new(BAD_URL_PATH_SPEC));
            }

            (Some(url), None) => input::mp4::Source::Url(url),
            (None, Some(path)) => input::mp4::Source::File(path.into()),
        };

        let queue_options = queue::InputOptions {
            required: required.unwrap_or(false),
            offset: offset_ms.map(|offset_ms| Duration::from_secs_f64(offset_ms / 1000.0)),
        };

        Ok(pipeline::RegisterInputOptions {
            input_options: input::InputOptions::Mp4(input::mp4::Mp4Options { source }),
            queue_options,
        })
    }
}
