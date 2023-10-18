use std::{sync::Arc, time::Duration};

use crate::queue::Queue;

use super::PipelineInput;
use compositor_common::{
    frame::YuvData,
    scene::{InputId, Resolution},
    Frame,
};
use ffmpeg_next::{
    codec::{Context, Id},
    frame::Video,
    media::Type,
};
use log::warn;

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
    pub fn new(queue: Arc<Queue>, input_options: Input::Opts, input_id: InputId) -> Self {
        let (input, packets) = Input::new(input_options);

        let parameters = input.decoder_parameters();

        std::thread::spawn(move || {
            let decoder = Context::from_parameters(parameters).unwrap();
            let decoder = decoder.decoder();
            let mut decoder = decoder.open_as(Into::<Id>::into(parameters.codec)).unwrap();

            let mut decoded_frame = ffmpeg_next::frame::Video::empty();
            let mut pts_offset = None;
            for packet in packets {
                decoder.send_packet(&packet).unwrap();

                while decoder.receive_frame(&mut decoded_frame).is_ok() {
                    let frame = match frame_from_av(&mut decoded_frame, &mut pts_offset) {
                        Ok(frame) => frame,
                        Err(err) => {
                            warn!("Error converting frame: {}", err);
                            continue;
                        }
                    };
                    queue.enqueue_frame(input_id.clone(), frame).unwrap();
                }
            }
        });

        Self { input }
    }

    pub fn input(&self) -> &Input {
        &self.input
    }
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
