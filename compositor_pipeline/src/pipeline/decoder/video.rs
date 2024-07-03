use compositor_render::{Frame, InputId};
use crossbeam_channel::{Receiver, Sender};

use crate::{
    error::InputInitError,
    pipeline::{types::EncodedChunk, VideoCodec},
    queue::PipelineEvent,
};

use super::VideoDecoderOptions;

mod ffmpeg_h264;

pub fn start_video_decoder_thread(
    options: VideoDecoderOptions,
    chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
    frame_sender: Sender<PipelineEvent<Frame>>,
    input_id: InputId,
) -> Result<(), InputInitError> {
    match options.codec {
        VideoCodec::H264 => {
            ffmpeg_h264::start_ffmpeg_decoder_thread(chunks_receiver, frame_sender, input_id)
        }
    }
}
