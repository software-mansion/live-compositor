use compositor_render::{Frame, InputId};
use crossbeam_channel::{Receiver, Sender};

use crate::{
    error::DecoderInitError,
    pipeline::{structs::EncodedChunk, VideoCodec},
    queue::PipelineEvent,
};

use self::ffmpeg_h264::H264FfmpegDecoder;

use super::VideoDecoderOptions;

mod ffmpeg_h264;

pub fn spawn_video_decoder(
    options: &VideoDecoderOptions,
    chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
    frame_sender: Sender<PipelineEvent<Frame>>,
    input_id: InputId,
) -> Result<(), DecoderInitError> {
    match options.codec {
        VideoCodec::H264 => H264FfmpegDecoder::spawn(chunks_receiver, frame_sender, input_id),
    }
}
