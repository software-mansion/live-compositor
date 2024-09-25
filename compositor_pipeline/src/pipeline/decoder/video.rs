use compositor_render::{Frame, InputId};
use crossbeam_channel::{Receiver, Sender};

use crate::{
    error::InputInitError,
    pipeline::{types::EncodedChunk, PipelineCtx, VideoCodec, VideoDecoder},
    queue::PipelineEvent,
};

use super::VideoDecoderOptions;

mod ffmpeg_h264;
mod vulkan_video;

pub fn start_video_decoder_thread(
    options: VideoDecoderOptions,
    pipeline_ctx: &PipelineCtx,
    chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
    frame_sender: Sender<PipelineEvent<Frame>>,
    input_id: InputId,
) -> Result<(), InputInitError> {
    match (options.codec, options.decoder) {
        (VideoCodec::H264, VideoDecoder::FFmpegH264) => {
            ffmpeg_h264::start_ffmpeg_decoder_thread(chunks_receiver, frame_sender, input_id)
        }

        (VideoCodec::H264, VideoDecoder::VulkanVideo) => {
            let Some(vulkan_ctx) = pipeline_ctx.vulkan_ctx.as_ref().map(|ctx| ctx.clone()) else {
                return Err(InputInitError::VulkanContextRequiredForVulkanDecoder);
            };

            vulkan_video::start_vulkan_video_decoder_thread(
                vulkan_ctx,
                chunks_receiver,
                frame_sender,
                input_id,
            )
        }
    }
}
