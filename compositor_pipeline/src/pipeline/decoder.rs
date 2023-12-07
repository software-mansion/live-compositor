use std::{sync::Arc, time::Duration};

use crate::{error::InputInitError, queue::Queue};

use super::PipelineInput;
use compositor_common::{
    error::ErrorStack,
    frame::YuvData,
    scene::{InputId, Resolution},
    Frame,
};
use ffmpeg_next::{
    codec::{Context, Id},
    ffi::AV_CODEC_FLAG2_CHUNKS,
    frame::Video,
    media::Type,
};
use log::{error, warn};
use rtp::packetizer::Depacketizer;

pub struct Decoder<Input: PipelineInput> {
    input: Input,
}

#[derive(Debug, Clone, Copy)]
pub struct DecoderParameters {
    pub codec: Codec,
}

#[derive(Debug, Clone, Copy)]
pub enum Codec {
    H264,
}

#[derive(Debug, thiserror::Error)]
enum DecoderError {
    #[error("Error converting frame: {0}")]
    FrameConversionError(String),
}

impl<Input: PipelineInput> Decoder<Input> {
    pub fn new(
        queue: Arc<Queue>,
        input_options: Input::Opts,
        input_id: InputId,
    ) -> Result<Self, InputInitError> {
        let (input, packets) = Input::new(input_options)?;
        let (init_result_sender, init_result_receiver) = crossbeam_channel::bounded(0);

        let parameters = input.decoder_parameters();

        std::thread::spawn(move || {
            let mut h264_depayloader = rtp::codecs::h264::H264Packet::default();

            let decoder = Context::from_parameters(parameters)
                .map_err(InputInitError::FfmpegError)
                .and_then(|mut decoder| {
                    // this flag allows us to send the packets in the form they come out of the depayloader
                    // wasted 6 hrs looking into this. I hate ffmpeg.
                    // and the bindings don't even expose `flags2` so we have to do the unsafe manually
                    unsafe {
                        (*decoder.as_mut_ptr()).flags2 |= AV_CODEC_FLAG2_CHUNKS;
                    }

                    let decoder = decoder.decoder();
                    decoder
                        .open_as(Into::<Id>::into(parameters.codec))
                        .map_err(InputInitError::FfmpegError)
                });

            let mut decoder = match decoder {
                Ok(decoder) => {
                    init_result_sender.send(Ok(())).unwrap();
                    decoder
                }
                Err(err) => {
                    init_result_sender.send(Err(err)).unwrap();
                    return;
                }
            };

            let mut decoded_frame = ffmpeg_next::frame::Video::empty();
            let mut pts_offset = None;
            for packet in packets {
                let av_packet = match packet_to_av(packet.clone(), &mut h264_depayloader) {
                    Ok(Some(packet)) => packet,
                    Ok(None) => continue,
                    Err(err) => {
                        warn!("Failed to depayload packet: {}", err);
                        continue;
                    }
                };

                match decoder.send_packet(&av_packet) {
                    Ok(()) => {}
                    Err(e) => {
                        warn!(
                            "Failed to send packet with sequence no {} to decoder: {}",
                            packet.header.sequence_number, e
                        );
                        continue;
                    }
                }

                while decoder.receive_frame(&mut decoded_frame).is_ok() {
                    let frame = match frame_from_av(&mut decoded_frame, &mut pts_offset) {
                        Ok(frame) => frame,
                        Err(err) => {
                            warn!("Dropping frame: {}", err);
                            continue;
                        }
                    };

                    if let Err(err) = queue.enqueue_frame(input_id.clone(), frame) {
                        error!(
                            "Failed to push frame: {}",
                            ErrorStack::new(&err).into_string()
                        );
                    }
                }
            }
        });

        init_result_receiver.recv().unwrap()?;

        Ok(Self { input })
    }

    pub fn input(&self) -> &Input {
        &self.input
    }
}

#[derive(Debug, thiserror::Error)]
enum DepayloadingError {
    #[error(transparent)]
    Rtp(#[from] rtp::Error),

    #[error("Payload type {0} indicates a different payload type than h264-encoded video")]
    BadPayloadType(u8),
}

const H264_DEFAULT_PAYLOAD_TYPE: u8 = 96;

fn packet_to_av(
    packet: rtp::packet::Packet,
    ctx: &mut rtp::codecs::h264::H264Packet,
) -> Result<Option<ffmpeg_next::packet::Packet>, DepayloadingError> {
    if packet.header.payload_type != H264_DEFAULT_PAYLOAD_TYPE {
        return Err(DepayloadingError::BadPayloadType(
            packet.header.payload_type,
        ));
    }

    let h264_packet = ctx.depacketize(&packet.payload)?;

    if h264_packet.is_empty() {
        return Ok(None);
    }

    let mut ffmpeg_packet = ffmpeg_next::packet::Packet::new(h264_packet.len());
    ffmpeg_packet
        .data_mut()
        .unwrap()
        .copy_from_slice(&h264_packet);

    ffmpeg_packet.set_pts(Some(packet.header.timestamp.into()));
    ffmpeg_packet.set_stream(0); // TODO: not sure if this is entirely correct

    Ok(Some(ffmpeg_packet))
}

fn frame_from_av(decoded: &mut Video, pts_offset: &mut Option<i64>) -> Result<Frame, DecoderError> {
    if decoded.format() != ffmpeg_next::format::pixel::Pixel::YUV420P {
        panic!("only YUV420P is supported");
    }
    let original_pts = decoded.pts();
    if let (Some(pts), None) = (decoded.pts(), &pts_offset) {
        *pts_offset = Some(-pts)
    }
    let pts = original_pts
        .map(|original_pts| original_pts + pts_offset.unwrap_or(0))
        .ok_or_else(|| DecoderError::FrameConversionError("missing pts".to_owned()))?;
    let pts = Duration::from_secs_f64(f64::max((pts as f64) / 90000.0, 0.0));
    Ok(Frame {
        data: YuvData {
            y_plane: copy_plane_from_av(decoded, 0),
            u_plane: copy_plane_from_av(decoded, 1),
            v_plane: copy_plane_from_av(decoded, 2),
        },
        resolution: Resolution {
            width: decoded.width().try_into().unwrap(),
            height: decoded.height().try_into().unwrap(),
        },
        pts,
    })
}

fn copy_plane_from_av(decoded: &Video, plane: usize) -> bytes::Bytes {
    let mut output_buffer = bytes::BytesMut::with_capacity(
        decoded.plane_width(plane) as usize * decoded.plane_height(plane) as usize,
    );

    decoded
        .data(plane)
        .chunks(decoded.stride(plane))
        .map(|chunk| &chunk[..decoded.plane_width(plane) as usize])
        .for_each(|chunk| output_buffer.extend_from_slice(chunk));

    output_buffer.freeze()
}

impl From<DecoderParameters> for ffmpeg_next::codec::Parameters {
    fn from(parameters: DecoderParameters) -> Self {
        match parameters.codec {
            Codec::H264 => {
                let mut parameters = ffmpeg_next::codec::Parameters::new();
                unsafe {
                    let parameters = &mut *parameters.as_mut_ptr();

                    parameters.codec_type = Type::Video.into();
                    parameters.codec_id = Id::H264.into();
                };
                parameters
            }
        }
    }
}

impl From<Codec> for ffmpeg_next::codec::Id {
    fn from(codec: Codec) -> Self {
        match codec {
            Codec::H264 => Id::H264,
        }
    }
}
