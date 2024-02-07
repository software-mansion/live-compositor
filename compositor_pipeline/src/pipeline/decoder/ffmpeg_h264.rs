use std::{sync::Arc, time::Duration};

use crate::{
    error::DecoderInitError,
    pipeline::structs::{EncodedChunk, EncodedChunkKind, VideoCodec},
    queue::Queue,
};

use compositor_render::{error::ErrorStack, Frame, InputId, Resolution, YuvData};
use crossbeam_channel::Receiver;
use ffmpeg_next::{
    codec::{Context, Id},
    ffi::AV_CODEC_FLAG2_CHUNKS,
    frame::Video,
    media::Type,
};
use log::{error, warn};

pub struct H264FfmpegDecoder;

impl H264FfmpegDecoder {
    pub fn new(
        chunks_receiver: Receiver<EncodedChunk>,
        queue: Arc<Queue>,
        input_id: InputId,
    ) -> Result<Self, DecoderInitError> {
        let (init_result_sender, init_result_receiver) = crossbeam_channel::bounded(0);

        let mut parameters = ffmpeg_next::codec::Parameters::new();
        unsafe {
            let parameters = &mut *parameters.as_mut_ptr();

            parameters.codec_type = Type::Video.into();
            parameters.codec_id = Id::H264.into();
        };

        std::thread::Builder::new()
            .name(format!("h264 ffmpeg decoder {}", input_id.0))
            .spawn(move || {
                let decoder = Context::from_parameters(parameters.clone())
                    .map_err(DecoderInitError::FfmpegError)
                    .and_then(|mut decoder| {
                        // this flag allows us to send the packets in the form they come out of the depayloader
                        // wasted 6 hrs looking into this. I hate ffmpeg.
                        // and the bindings don't even expose `flags2` so we have to do the unsafe manually
                        unsafe {
                            (*decoder.as_mut_ptr()).flags2 |= AV_CODEC_FLAG2_CHUNKS;
                        }

                        let decoder = decoder.decoder();
                        decoder
                            .open_as(Into::<Id>::into(parameters.id()))
                            .map_err(DecoderInitError::FfmpegError)
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
                for chunk in chunks_receiver {
                    if chunk.kind != EncodedChunkKind::Video(VideoCodec::H264) {
                        error!(
                            "H264 decoder received chunk of wrong kind: {:?}",
                            chunk.kind
                        );
                        continue;
                    }

                    let av_packet: ffmpeg_next::Packet = match chunk_to_av(chunk) {
                        Ok(packet) => packet,
                        Err(err) => {
                            warn!("Dropping frame: {}", err);
                            continue;
                        }
                    };

                    match decoder.send_packet(&av_packet) {
                        Ok(()) => {}
                        Err(e) => {
                            warn!("Failed to send a packet to decoder: {}", e);
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

                        if let Err(err) = queue.enqueue_video_frame(input_id.clone(), frame) {
                            error!(
                                "Failed to push frame: {}",
                                ErrorStack::new(&err).into_string()
                            );
                        }
                    }
                }
            })
            .unwrap();

        init_result_receiver.recv().unwrap()?;

        Ok(Self)
    }
}

#[derive(Debug, thiserror::Error)]
enum DecoderChunkConversionError {
    #[error(
        "Cannot send a chunk of kind {0:?} to the decoder. The decoder only handles H264-encoded video."
    )]
    BadPayloadType(EncodedChunkKind),
}

fn chunk_to_av(chunk: EncodedChunk) -> Result<ffmpeg_next::Packet, DecoderChunkConversionError> {
    if chunk.kind != EncodedChunkKind::Video(VideoCodec::H264) {
        return Err(DecoderChunkConversionError::BadPayloadType(chunk.kind));
    }

    let mut packet = ffmpeg_next::Packet::new(chunk.data.len());

    packet.data_mut().unwrap().copy_from_slice(&chunk.data);
    packet.set_pts(Some(chunk.pts));
    packet.set_dts(chunk.dts);

    Ok(packet)
}

#[derive(Debug, thiserror::Error)]
enum DecoderFrameConversionError {
    #[error("Error converting frame: {0}")]
    FrameConversionError(String),
}

fn frame_from_av(
    decoded: &mut Video,
    pts_offset: &mut Option<i64>,
) -> Result<Frame, DecoderFrameConversionError> {
    if decoded.format() != ffmpeg_next::format::pixel::Pixel::YUV420P {
        panic!("only YUV420P is supported");
    }
    let original_pts = decoded.pts();
    if let (Some(pts), None) = (decoded.pts(), &pts_offset) {
        *pts_offset = Some(-pts)
    }
    let pts = original_pts
        .map(|original_pts| original_pts + pts_offset.unwrap_or(0))
        .ok_or_else(|| {
            DecoderFrameConversionError::FrameConversionError("missing pts".to_owned())
        })?;
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
