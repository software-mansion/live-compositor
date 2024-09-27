use compositor_render::{Frame, InputId};
use crossbeam_channel::{Receiver, Sender};

use crate::{
    error::InputInitError,
    pipeline::{types::EncodedChunk, PipelineCtx, VideoCodec, VideoDecoder},
    queue::PipelineEvent,
};

use super::VideoDecoderOptions;

mod ffmpeg_h264;
#[cfg(feature = "vk-video")]
mod vulkan_video;

pub fn start_video_decoder_thread(
    options: VideoDecoderOptions,
    pipeline_ctx: &PipelineCtx,
    chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
    frame_sender: Sender<PipelineEvent<Frame>>,
    input_id: InputId,
) -> Result<(), InputInitError> {
    match (options.codec, options.decoder) {
        (VideoCodec::H264, VideoDecoder::FFmpegH264) => ffmpeg_h264::start_ffmpeg_decoder_thread(
            pipeline_ctx,
            chunks_receiver,
            frame_sender,
            input_id,
        ),

        #[cfg(feature = "vk-video")]
        (VideoCodec::H264, VideoDecoder::VulkanVideo) => {
            vulkan_video::start_vulkan_video_decoder_thread(
                pipeline_ctx,
                chunks_receiver,
                frame_sender,
                input_id,
            )
        }
    }
}
