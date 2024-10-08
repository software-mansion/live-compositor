use ash::vk;
use h264_reader::nal::{pps::PicParameterSet, sps::SeqParameterSet};
use images::DecodingImages;
use parameters::VideoSessionParametersManager;

use super::{
    CommandBuffer, Fence, H264ProfileInfo, SeqParameterSetExt, VideoSession, VulkanCtx,
    VulkanDecoderError,
};

mod images;
mod parameters;

pub(super) struct VideoSessionResources<'a> {
    pub(crate) video_session: VideoSession,
    pub(crate) parameters_manager: VideoSessionParametersManager,
    pub(crate) decoding_images: DecodingImages<'a>,
}

impl VideoSessionResources<'_> {
    pub(crate) fn new_from_sps(
        vulkan_ctx: &VulkanCtx,
        decode_buffer: &CommandBuffer,
        sps: &SeqParameterSet,
        fence_memory_barrier_completed: &Fence,
    ) -> Result<Self, VulkanDecoderError> {
        let profile = H264ProfileInfo::decode_h264_yuv420();

        let width = sps.width()?;
        let height = sps.height()?;

        let max_coded_extent = vk::Extent2D { width, height };
        // +1 for current frame
        let max_dpb_slots = sps.max_num_ref_frames + 1;
        let max_active_references = sps.max_num_ref_frames;

        let video_session = VideoSession::new(
            vulkan_ctx,
            &profile.profile_info,
            max_coded_extent,
            max_dpb_slots,
            max_active_references,
            &vulkan_ctx.video_capabilities.std_header_version,
        )?;

        let mut parameters_manager =
            VideoSessionParametersManager::new(vulkan_ctx, video_session.session)?;

        parameters_manager.put_sps(sps)?;

        let decoding_images = Self::new_decoding_images(
            vulkan_ctx,
            max_coded_extent,
            max_dpb_slots,
            decode_buffer,
            fence_memory_barrier_completed,
        )?;

        Ok(VideoSessionResources {
            video_session,
            parameters_manager,
            decoding_images,
        })
    }

    pub(crate) fn process_sps(
        &mut self,
        vulkan_ctx: &VulkanCtx,
        decode_buffer: &CommandBuffer,
        sps: &SeqParameterSet,
        fence_memory_barrier_completed: &Fence,
    ) -> Result<(), VulkanDecoderError> {
        let profile = H264ProfileInfo::decode_h264_yuv420();

        let width = sps.width()?;
        let height = sps.height()?;

        let max_coded_extent = vk::Extent2D { width, height };
        // +1 for current frame
        let max_dpb_slots = sps.max_num_ref_frames + 1;
        let max_active_references = sps.max_num_ref_frames;

        if self.video_session.max_coded_extent.width >= width
            && self.video_session.max_coded_extent.height >= height
            && self.video_session.max_dpb_slots >= max_dpb_slots
        {
            // no need to change the session
            self.parameters_manager.put_sps(sps)?;
            return Ok(());
        }

        self.video_session = VideoSession::new(
            vulkan_ctx,
            &profile.profile_info,
            max_coded_extent,
            max_dpb_slots,
            max_active_references,
            &vulkan_ctx.video_capabilities.std_header_version,
        )?;

        self.parameters_manager
            .change_session(self.video_session.session)?;
        self.parameters_manager.put_sps(sps)?;

        self.decoding_images = Self::new_decoding_images(
            vulkan_ctx,
            max_coded_extent,
            max_dpb_slots,
            decode_buffer,
            fence_memory_barrier_completed,
        )?;

        Ok(())
    }

    pub(crate) fn process_pps(&mut self, pps: &PicParameterSet) -> Result<(), VulkanDecoderError> {
        self.parameters_manager.put_pps(pps)
    }

    fn new_decoding_images<'a>(
        vulkan_ctx: &VulkanCtx,
        max_coded_extent: vk::Extent2D,
        max_dpb_slots: u32,
        decode_buffer: &CommandBuffer,
        fence_memory_barrier_completed: &Fence,
    ) -> Result<DecodingImages<'a>, VulkanDecoderError> {
        let profile = H264ProfileInfo::decode_h264_yuv420();

        // FIXME: usually, sps arrives either at the start of the stream (when all spses are sent
        // at the begginning of the stream) or right before an IDR. It is however possible for an
        // sps nal to arrive in between P-frames. This would cause us to loose the reference
        // pictures we need to decode the stream until we receive a new IDR. Don't know if this is
        // an issue worth fixing, I don't think I ever saw a stream like this.
        let (decoding_images, memory_barrier) = DecodingImages::new(
            vulkan_ctx,
            profile,
            &vulkan_ctx.h264_dpb_format_properties,
            &vulkan_ctx.h264_dst_format_properties,
            max_coded_extent,
            max_dpb_slots,
        )?;

        decode_buffer.begin()?;

        unsafe {
            vulkan_ctx.device.cmd_pipeline_barrier2(
                **decode_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&memory_barrier),
            );
        }

        decode_buffer.end()?;

        vulkan_ctx.queues.h264_decode.submit(
            decode_buffer,
            &[],
            &[],
            Some(**fence_memory_barrier_completed),
        )?;

        // TODO: this shouldn't be a fence
        fence_memory_barrier_completed.wait_and_reset(u64::MAX)?;

        Ok(decoding_images)
    }

    pub(crate) fn free_reference_picture(&mut self, i: usize) -> Result<(), VulkanDecoderError> {
        self.decoding_images.free_reference_picture(i)
    }
}
