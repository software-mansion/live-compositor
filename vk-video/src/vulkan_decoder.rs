use std::sync::Arc;

use ash::vk;

use h264_reader::nal::{pps::PicParameterSet, sps::SeqParameterSet};
use tracing::error;
use wrappers::*;

use crate::parser::{DecodeInformation, DecoderInstruction, ReferenceId};

mod parameter_sets;
mod vulkan_ctx;
mod wrappers;

pub use vulkan_ctx::*;

const MACROBLOCK_SIZE: u32 = 16;

pub struct VulkanDecoder<'a> {
    vulkan_ctx: Arc<VulkanCtx>,
    video_session_resources: Option<VideoSessionResources<'a>>,
    command_buffers: CommandBuffers,
    _command_pools: CommandPools,
    sync_structures: SyncStructures,
    reference_id_to_dpb_slot_index: std::collections::HashMap<ReferenceId, usize>,
    decode_query_pool: Option<DecodeQueryPool>,
}

struct SyncStructures {
    sem_decode_done: Semaphore,
    fence_transfer_done: Fence,
    fence_memory_barrier_completed: Fence,
}

struct CommandBuffers {
    decode_buffer: CommandBuffer,
    gpu_to_mem_transfer_buffer: CommandBuffer,
    vulkan_to_wgpu_transfer_buffer: CommandBuffer,
}

struct VideoSessionResources<'a> {
    video_session: VideoSession,
    parameters_manager: VideoSessionParametersManager,
    decoding_images: DecodingImages<'a>,
}

/// this cannot outlive the image and semaphore it borrows, but it seems impossible to encode that
/// in the lifetimes
struct DecodeOutput {
    image: vk::Image,
    dimensions: vk::Extent2D,
    current_layout: vk::ImageLayout,
    layer: u32,
    wait_semaphore: vk::Semaphore,
    _input_buffer: Buffer,
}

#[derive(Debug, thiserror::Error)]
pub enum VulkanDecoderError {
    #[error("Vulkan error: {0}")]
    VkError(#[from] vk::Result),

    #[error("Cannot find enough memory of the right type on the deivce")]
    NoMemory,

    #[error("The decoder instruction is not supported: {0:?}")]
    DecoderInstructionNotSupported(Box<DecoderInstruction>),

    #[error("Setting the frame cropping flag in sps is not supported")]
    FrameCroppingNotSupported,

    #[error("Bitstreams that contain fields rather than frames are not supported")]
    FieldsNotSupported,

    #[error("Scaling lists are not supported")]
    ScalingListsNotSupported,

    #[error("A NALU requiring a session received before a session was created (probably before receiving first SPS)")]
    NoSession,

    #[error("A slot in the Decoded Pictures Buffer was requested, but all slots are taken")]
    NoFreeSlotsInDpb,

    #[error("A picture which is not in the decoded pictures buffer was requested as a reference picture")]
    NonExistantReferenceRequested,

    #[error("A vulkan decode operation failed with code {0:?}")]
    DecodeOperationFailed(vk::QueryResultStatusKHR),

    #[error(transparent)]
    VulkanCtxError(#[from] VulkanCtxError),
}

impl<'a> VulkanDecoder<'a> {
    pub fn new(vulkan_ctx: Arc<VulkanCtx>) -> Result<Self, VulkanDecoderError> {
        let decode_pool = Arc::new(CommandPool::new(
            vulkan_ctx.device.clone(),
            vulkan_ctx.queues.h264_decode.idx,
        )?);

        let transfer_pool = Arc::new(CommandPool::new(
            vulkan_ctx.device.clone(),
            vulkan_ctx.queues.transfer.idx,
        )?);

        let decode_buffer = CommandBuffer::new_primary(decode_pool.clone())?;

        let gpu_to_mem_transfer_buffer = CommandBuffer::new_primary(transfer_pool.clone())?;

        let vulkan_to_wgpu_transfer_buffer = CommandBuffer::new_primary(transfer_pool.clone())?;

        let command_pools = CommandPools {
            _decode_pool: decode_pool,
            _transfer_pool: transfer_pool,
        };

        let sync_structures = SyncStructures {
            sem_decode_done: Semaphore::new(vulkan_ctx.device.clone())?,
            fence_transfer_done: Fence::new(vulkan_ctx.device.clone(), false)?,
            fence_memory_barrier_completed: Fence::new(vulkan_ctx.device.clone(), false)?,
        };

        let decode_query_pool = if vulkan_ctx
            .queues
            .h264_decode
            .supports_result_status_queries()
        {
            Some(DecodeQueryPool::new(
                vulkan_ctx.device.clone(),
                H264ProfileInfo::decode_h264_yuv420().profile_info,
            )?)
        } else {
            None
        };

        Ok(Self {
            vulkan_ctx,
            video_session_resources: None,
            _command_pools: command_pools,
            command_buffers: CommandBuffers {
                decode_buffer,
                gpu_to_mem_transfer_buffer,
                vulkan_to_wgpu_transfer_buffer,
            },
            sync_structures,
            decode_query_pool,
            reference_id_to_dpb_slot_index: Default::default(),
        })
    }
}

impl VulkanDecoder<'_> {
    pub fn decode_to_bytes(
        &mut self,
        decoder_instructions: &[DecoderInstruction],
    ) -> Result<Vec<Vec<u8>>, VulkanDecoderError> {
        let mut result = Vec::new();
        for instruction in decoder_instructions {
            if let Some(output) = self.decode(instruction)? {
                result.push(self.download_output(output)?)
            }
        }

        Ok(result)
    }

    pub fn decode_to_wgpu_textures(
        &mut self,
        decoder_instructions: &[DecoderInstruction],
    ) -> Result<Vec<wgpu::Texture>, VulkanDecoderError> {
        let mut result = Vec::new();
        for instruction in decoder_instructions {
            if let Some(output) = self.decode(instruction)? {
                result.push(self.output_to_wgpu_texture(output)?)
            }
        }

        Ok(result)
    }

    fn decode(
        &mut self,
        instruction: &DecoderInstruction,
    ) -> Result<Option<DecodeOutput>, VulkanDecoderError> {
        match instruction {
            DecoderInstruction::Decode { .. } => {
                return Err(VulkanDecoderError::DecoderInstructionNotSupported(
                    Box::new(instruction.clone()),
                ))
            }

            DecoderInstruction::DecodeAndStoreAs {
                decode_info,
                reference_id,
            } => {
                return self
                    .process_reference_p_frame(decode_info, *reference_id)
                    .map(Option::Some)
            }

            DecoderInstruction::Idr {
                decode_info,
                reference_id,
            } => {
                return self
                    .process_idr(decode_info, *reference_id)
                    .map(Option::Some)
            }

            DecoderInstruction::Drop { reference_ids } => {
                for reference_id in reference_ids {
                    match self.reference_id_to_dpb_slot_index.remove(reference_id) {
                        Some(dpb_idx) => self
                            .video_session_resources
                            .as_mut()
                            .map(|s| s.decoding_images.free_reference_picture(dpb_idx)),
                        None => return Err(VulkanDecoderError::NonExistantReferenceRequested),
                    };
                }
            }

            DecoderInstruction::Sps(sps) => self.process_sps(sps)?,

            DecoderInstruction::Pps(pps) => self.process_pps(pps)?,
        }

        Ok(None)
    }

    fn process_sps(&mut self, sps: &SeqParameterSet) -> Result<(), VulkanDecoderError> {
        let profile = H264ProfileInfo::decode_h264_yuv420();

        let width = match sps.frame_cropping {
            None => (sps.pic_width_in_mbs_minus1 + 1) * MACROBLOCK_SIZE,
            Some(_) => return Err(VulkanDecoderError::FrameCroppingNotSupported),
        };

        let height = match sps.frame_mbs_flags {
            h264_reader::nal::sps::FrameMbsFlags::Frames => {
                (sps.pic_height_in_map_units_minus1 + 1) * MACROBLOCK_SIZE
            }
            h264_reader::nal::sps::FrameMbsFlags::Fields { .. } => {
                return Err(VulkanDecoderError::FieldsNotSupported)
            }
        };

        let max_coded_extent = vk::Extent2D { width, height };
        // +1 for current frame
        let max_dpb_slots = sps.max_num_ref_frames + 1;
        let max_active_references = sps.max_num_ref_frames;

        if let Some(VideoSessionResources {
            video_session,
            parameters_manager: parameters,
            ..
        }) = &mut self.video_session_resources
        {
            if video_session.max_coded_extent.width >= width
                && video_session.max_coded_extent.height >= height
                && video_session.max_dpb_slots >= max_dpb_slots
            {
                // no need to change the session
                parameters.put_sps(sps)?;
                return Ok(());
            }
        }

        let video_session = VideoSession::new(
            &self.vulkan_ctx,
            &profile.profile_info,
            max_coded_extent,
            max_dpb_slots,
            max_active_references,
            &self.vulkan_ctx.video_capabilities.std_header_version,
        )?;

        let parameters = self
            .video_session_resources
            .take()
            .map(|r| r.parameters_manager);

        let mut parameters = match parameters {
            Some(mut parameters) => {
                parameters.change_session(video_session.session)?;
                parameters
            }
            None => VideoSessionParametersManager::new(&self.vulkan_ctx, video_session.session)?,
        };

        parameters.put_sps(sps)?;

        // FIXME: usually, sps arrives either at the start of the stream (when all spses are sent
        // at the begginning of the stream) or right before an IDR. It is however possible for an
        // sps nal to arrive in between P-frames. This would cause us to loose the reference
        // pictures we need to decode the stream until we receive a new IDR. Don't know if this is
        // an issue worth fixing, I don't think I ever saw a stream like this.
        let (decoding_images, memory_barrier) = DecodingImages::new(
            &self.vulkan_ctx,
            profile,
            &self.vulkan_ctx.h264_dpb_format_properties,
            &self.vulkan_ctx.h264_dst_format_properties,
            max_coded_extent,
            max_dpb_slots,
        )?;

        self.command_buffers.decode_buffer.begin()?;

        unsafe {
            self.vulkan_ctx.device.cmd_pipeline_barrier2(
                *self.command_buffers.decode_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&memory_barrier),
            );
        }

        self.command_buffers.decode_buffer.end()?;

        self.command_buffers.decode_buffer.submit(
            *self.vulkan_ctx.queues.h264_decode.queue.lock().unwrap(),
            &[],
            &[],
            Some(*self.sync_structures.fence_memory_barrier_completed),
        )?;

        // TODO: this shouldn't be a fence
        self.sync_structures
            .fence_memory_barrier_completed
            .wait_and_reset(u64::MAX)?;

        self.video_session_resources = Some(VideoSessionResources {
            video_session,
            parameters_manager: parameters,
            decoding_images,
        });

        Ok(())
    }

    fn process_pps(&mut self, pps: &PicParameterSet) -> Result<(), VulkanDecoderError> {
        self.video_session_resources
            .as_mut()
            .map(|r| &mut r.parameters_manager)
            .ok_or(VulkanDecoderError::NoSession)?
            .put_pps(pps)?;

        Ok(())
    }

    fn pad_size_to_alignment(size: u64, align: u64) -> u64 {
        if size % align == 0 {
            size
        } else {
            (size + align) / align * align
        }
    }

    fn process_idr(
        &mut self,
        decode_information: &DecodeInformation,
        reference_id: ReferenceId,
    ) -> Result<DecodeOutput, VulkanDecoderError> {
        self.do_decode(decode_information, reference_id, true, true)
    }

    fn process_reference_p_frame(
        &mut self,
        decode_information: &DecodeInformation,
        reference_id: ReferenceId,
    ) -> Result<DecodeOutput, VulkanDecoderError> {
        self.do_decode(decode_information, reference_id, false, true)
    }

    fn do_decode(
        &mut self,
        decode_information: &DecodeInformation,
        reference_id: ReferenceId,
        is_idr: bool,
        is_reference: bool,
    ) -> Result<DecodeOutput, VulkanDecoderError> {
        // upload data to a buffer
        let size = Self::pad_size_to_alignment(
            decode_information.rbsp_bytes.len() as u64,
            self.vulkan_ctx
                .video_capabilities
                .min_bitstream_buffer_offset_alignment,
        );

        let decode_buffer =
            self.upload_decode_data_to_buffer(&decode_information.rbsp_bytes, size)?;

        // decode
        let video_session_resources = self
            .video_session_resources
            .as_mut()
            .ok_or(VulkanDecoderError::NoSession)?;

        // IDR - remove all reference picures
        if is_idr {
            video_session_resources
                .decoding_images
                .reset_all_allocations();

            self.reference_id_to_dpb_slot_index = Default::default();
        }

        // begin video coding
        self.command_buffers.decode_buffer.begin()?;

        let memory_barrier = vk::MemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::VIDEO_DECODE_KHR)
            .src_access_mask(vk::AccessFlags2::VIDEO_DECODE_WRITE_KHR)
            .dst_stage_mask(vk::PipelineStageFlags2::VIDEO_DECODE_KHR)
            .dst_access_mask(
                vk::AccessFlags2::VIDEO_DECODE_READ_KHR | vk::AccessFlags2::VIDEO_DECODE_WRITE_KHR,
            );

        unsafe {
            self.vulkan_ctx.device.cmd_pipeline_barrier2(
                *self.command_buffers.decode_buffer,
                &vk::DependencyInfo::default().memory_barriers(&[memory_barrier]),
            )
        };

        if let Some(pool) = self.decode_query_pool.as_ref() {
            pool.reset(*self.command_buffers.decode_buffer);
        }

        let reference_slots = video_session_resources
            .decoding_images
            .reference_slot_info();

        let begin_info = vk::VideoBeginCodingInfoKHR::default()
            .video_session(video_session_resources.video_session.session)
            .video_session_parameters(video_session_resources.parameters_manager.parameters())
            .reference_slots(&reference_slots);

        unsafe {
            self.vulkan_ctx
                .device
                .video_queue_ext
                .cmd_begin_video_coding_khr(*self.command_buffers.decode_buffer, &begin_info)
        };

        // IDR - issue the reset command to the video session
        if is_idr {
            let control_info = vk::VideoCodingControlInfoKHR::default()
                .flags(vk::VideoCodingControlFlagsKHR::RESET);

            unsafe {
                self.vulkan_ctx
                    .device
                    .video_queue_ext
                    .cmd_control_video_coding_khr(
                        *self.command_buffers.decode_buffer,
                        &control_info,
                    )
            };
        }

        // allocate a new reference picture and fill out the forms to get it set up
        let new_reference_slot_index = video_session_resources
            .decoding_images
            .allocate_reference_picture()?;

        let new_reference_slot_std_reference_info = decode_information.picture_info.into();
        let mut new_reference_slot_dpb_slot_info = vk::VideoDecodeH264DpbSlotInfoKHR::default()
            .std_reference_info(&new_reference_slot_std_reference_info);

        let new_reference_slot_video_picture_resource_info = video_session_resources
            .decoding_images
            .video_resource_info(new_reference_slot_index)
            .unwrap();

        let setup_reference_slot = vk::VideoReferenceSlotInfoKHR::default()
            .picture_resource(new_reference_slot_video_picture_resource_info)
            .slot_index(new_reference_slot_index as i32)
            .push_next(&mut new_reference_slot_dpb_slot_info);

        // prepare the reference list
        let reference_slots = video_session_resources
            .decoding_images
            .reference_slot_info();

        let references_std_ref_info = Self::prepare_references_std_ref_info(decode_information);

        let mut references_dpb_slot_info =
            Self::prepare_references_dpb_slot_info(&references_std_ref_info);

        let pic_reference_slots = Self::prepare_reference_list_slot_info(
            &self.reference_id_to_dpb_slot_index,
            &reference_slots,
            &mut references_dpb_slot_info,
            decode_information,
        )?;

        // prepare the decode target picture
        let std_picture_info = vk::native::StdVideoDecodeH264PictureInfo {
            flags: vk::native::StdVideoDecodeH264PictureInfoFlags {
                _bitfield_align_1: [],
                __bindgen_padding_0: [0; 3],
                _bitfield_1: vk::native::StdVideoDecodeH264PictureInfoFlags::new_bitfield_1(
                    matches!(
                        decode_information.header.field_pic,
                        h264_reader::nal::slice::FieldPic::Field(..)
                    )
                    .into(),
                    is_idr.into(),
                    is_idr.into(),
                    0,
                    is_reference.into(),
                    0,
                ),
            },
            PicOrderCnt: decode_information.picture_info.PicOrderCnt,
            seq_parameter_set_id: decode_information.sps_id,
            pic_parameter_set_id: decode_information.pps_id,
            frame_num: decode_information.header.frame_num,
            idr_pic_id: decode_information
                .header
                .idr_pic_id
                .map(|a| a as u16)
                .unwrap_or(0),
            reserved1: 0,
            reserved2: 0,
        };

        let slice_offsets = decode_information
            .slice_indices
            .iter()
            .map(|&x| x as u32)
            .collect::<Vec<_>>();

        let mut decode_h264_picture_info = vk::VideoDecodeH264PictureInfoKHR::default()
            .std_picture_info(&std_picture_info)
            .slice_offsets(&slice_offsets);

        let dst_picture_resource_info = match &video_session_resources.decoding_images.dst_image {
            Some(image) => image.video_resource_info[0],
            None => *new_reference_slot_video_picture_resource_info,
        };

        // these 3 veriables are for copying the result later
        let (dst_image, dst_image_layout, dst_layer) =
            match &video_session_resources.decoding_images.dst_image {
                Some(image) => (**image.image, vk::ImageLayout::VIDEO_DECODE_DST_KHR, 0),
                None => (
                    **video_session_resources.decoding_images.dpb_image.image,
                    vk::ImageLayout::VIDEO_DECODE_DPB_KHR,
                    new_reference_slot_index,
                ),
            };

        // fill out the final struct and issue the command
        let decode_info = vk::VideoDecodeInfoKHR::default()
            .src_buffer(*decode_buffer)
            .src_buffer_offset(0)
            .src_buffer_range(size)
            .dst_picture_resource(dst_picture_resource_info)
            .setup_reference_slot(&setup_reference_slot)
            .reference_slots(&pic_reference_slots)
            .push_next(&mut decode_h264_picture_info);

        if let Some(pool) = self.decode_query_pool.as_ref() {
            pool.begin_query(*self.command_buffers.decode_buffer);
        }

        unsafe {
            self.vulkan_ctx
                .device
                .video_decode_queue_ext
                .cmd_decode_video_khr(*self.command_buffers.decode_buffer, &decode_info)
        };

        if let Some(pool) = self.decode_query_pool.as_ref() {
            pool.end_query(*self.command_buffers.decode_buffer);
        }

        unsafe {
            self.vulkan_ctx
                .device
                .video_queue_ext
                .cmd_end_video_coding_khr(
                    *self.command_buffers.decode_buffer,
                    &vk::VideoEndCodingInfoKHR::default(),
                )
        };

        self.command_buffers.decode_buffer.end()?;

        self.command_buffers.decode_buffer.submit(
            *self.vulkan_ctx.queues.h264_decode.queue.lock().unwrap(),
            &[],
            &[(
                *self.sync_structures.sem_decode_done,
                vk::PipelineStageFlags2::VIDEO_DECODE_KHR,
            )],
            None,
        )?;

        // after the decode save the new reference picture
        self.reference_id_to_dpb_slot_index
            .insert(reference_id, new_reference_slot_index);

        // TODO: those are not the real dimensions of the image. the real dimensions should be
        // calculated from the sps
        let dimensions = video_session_resources.video_session.max_coded_extent;

        Ok(DecodeOutput {
            image: dst_image,
            wait_semaphore: *self.sync_structures.sem_decode_done,
            layer: dst_layer as u32,
            current_layout: dst_image_layout,
            dimensions,
            _input_buffer: decode_buffer,
        })
    }

    fn output_to_wgpu_texture(
        &self,
        decode_output: DecodeOutput,
    ) -> Result<wgpu::Texture, VulkanDecoderError> {
        let copy_extent = vk::Extent3D {
            width: decode_output.dimensions.width,
            height: decode_output.dimensions.height,
            depth: 1,
        };

        let queue_indices = [
            self.vulkan_ctx.queues.transfer.idx as u32,
            self.vulkan_ctx.queues.wgpu.idx as u32,
        ];

        let create_info = vk::ImageCreateInfo::default()
            .flags(vk::ImageCreateFlags::MUTABLE_FORMAT)
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::G8_B8R8_2PLANE_420_UNORM)
            .extent(copy_extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(
                vk::ImageUsageFlags::SAMPLED
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::TRANSFER_SRC,
            )
            .sharing_mode(vk::SharingMode::CONCURRENT)
            .queue_family_indices(&queue_indices)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = Arc::new(Image::new(self.vulkan_ctx.allocator.clone(), &create_info)?);

        self.command_buffers
            .vulkan_to_wgpu_transfer_buffer
            .begin()?;

        let memory_barrier_src = vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::NONE)
            .src_access_mask(vk::AccessFlags2::NONE)
            .dst_stage_mask(vk::PipelineStageFlags2::COPY)
            .dst_access_mask(vk::AccessFlags2::TRANSFER_READ)
            .old_layout(decode_output.current_layout)
            .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(decode_output.image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: decode_output.layer,
                layer_count: 1,
            });

        let memory_barrier_dst = vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::NONE)
            .src_access_mask(vk::AccessFlags2::NONE)
            .dst_stage_mask(vk::PipelineStageFlags2::COPY)
            .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(**image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        unsafe {
            self.vulkan_ctx.device.cmd_pipeline_barrier2(
                *self.command_buffers.vulkan_to_wgpu_transfer_buffer,
                &vk::DependencyInfo::default()
                    .image_memory_barriers(&[memory_barrier_src, memory_barrier_dst]),
            )
        };

        let copy_info = [
            vk::ImageCopy::default()
                .src_subresource(vk::ImageSubresourceLayers {
                    base_array_layer: decode_output.layer,
                    mip_level: 0,
                    layer_count: 1,
                    aspect_mask: vk::ImageAspectFlags::PLANE_0,
                })
                .src_offset(vk::Offset3D::default())
                .dst_subresource(vk::ImageSubresourceLayers {
                    base_array_layer: 0,
                    mip_level: 0,
                    layer_count: 1,
                    aspect_mask: vk::ImageAspectFlags::PLANE_0,
                })
                .dst_offset(vk::Offset3D::default())
                .extent(copy_extent),
            vk::ImageCopy::default()
                .src_subresource(vk::ImageSubresourceLayers {
                    base_array_layer: decode_output.layer,
                    mip_level: 0,
                    layer_count: 1,
                    aspect_mask: vk::ImageAspectFlags::PLANE_1,
                })
                .src_offset(vk::Offset3D::default())
                .dst_subresource(vk::ImageSubresourceLayers {
                    base_array_layer: 0,
                    mip_level: 0,
                    layer_count: 1,
                    aspect_mask: vk::ImageAspectFlags::PLANE_1,
                })
                .dst_offset(vk::Offset3D::default())
                .extent(vk::Extent3D {
                    width: copy_extent.width / 2,
                    height: copy_extent.height / 2,
                    ..copy_extent
                }),
        ];

        unsafe {
            self.vulkan_ctx.device.cmd_copy_image(
                *self.command_buffers.vulkan_to_wgpu_transfer_buffer,
                decode_output.image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                **image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &copy_info,
            );
        }

        let memory_barrier_src = memory_barrier_src
            .src_stage_mask(vk::PipelineStageFlags2::COPY)
            .src_access_mask(vk::AccessFlags2::TRANSFER_READ)
            .dst_stage_mask(vk::PipelineStageFlags2::NONE)
            .dst_access_mask(vk::AccessFlags2::NONE)
            .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .new_layout(decode_output.current_layout);

        let memory_barrier_dst = memory_barrier_dst
            .src_stage_mask(vk::PipelineStageFlags2::COPY)
            .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::NONE)
            .dst_access_mask(vk::AccessFlags2::NONE)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::GENERAL);

        unsafe {
            self.vulkan_ctx.device.cmd_pipeline_barrier2(
                *self.command_buffers.vulkan_to_wgpu_transfer_buffer,
                &vk::DependencyInfo::default()
                    .image_memory_barriers(&[memory_barrier_src, memory_barrier_dst]),
            )
        };

        self.command_buffers.vulkan_to_wgpu_transfer_buffer.end()?;

        self.command_buffers.vulkan_to_wgpu_transfer_buffer.submit(
            *self.vulkan_ctx.queues.transfer.queue.lock().unwrap(),
            &[(
                decode_output.wait_semaphore,
                vk::PipelineStageFlags2::TOP_OF_PIPE,
            )],
            &[],
            Some(*self.sync_structures.fence_transfer_done),
        )?;

        self.sync_structures
            .fence_transfer_done
            .wait_and_reset(u64::MAX)?;

        let result = self
            .decode_query_pool
            .as_ref()
            .map(|pool| pool.get_result_blocking());

        if let Some(result) = result {
            let result = result?;
            if result.as_raw() < 0 {
                return Err(VulkanDecoderError::DecodeOperationFailed(result));
            }
        }

        let hal_texture = unsafe {
            wgpu::hal::vulkan::Device::texture_from_raw(
                **image,
                &wgpu::hal::TextureDescriptor {
                    label: Some("vulkan video output texture"),
                    usage: wgpu::hal::TextureUses::RESOURCE
                        | wgpu::hal::TextureUses::COPY_DST
                        | wgpu::hal::TextureUses::COPY_SRC,
                    memory_flags: wgpu::hal::MemoryFlags::empty(),
                    size: wgpu::Extent3d {
                        width: copy_extent.width,
                        height: copy_extent.height,
                        depth_or_array_layers: copy_extent.depth,
                    },
                    dimension: wgpu::TextureDimension::D2,
                    sample_count: 1,
                    view_formats: Vec::new(),
                    format: wgpu::TextureFormat::NV12,
                    mip_level_count: 1,
                },
                Some(Box::new(image.clone())),
            )
        };

        let wgpu_texture = unsafe {
            self.vulkan_ctx
                .wgpu_ctx
                .device
                .create_texture_from_hal::<wgpu::hal::vulkan::Api>(
                    hal_texture,
                    &wgpu::TextureDescriptor {
                        label: Some("vulkan video output texture"),
                        usage: wgpu::TextureUsages::COPY_DST
                            | wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::COPY_SRC,
                        size: wgpu::Extent3d {
                            width: copy_extent.width,
                            height: copy_extent.height,
                            depth_or_array_layers: copy_extent.depth,
                        },
                        dimension: wgpu::TextureDimension::D2,
                        sample_count: 1,
                        view_formats: &[],
                        format: wgpu::TextureFormat::NV12,
                        mip_level_count: 1,
                    },
                )
        };

        Ok(wgpu_texture)
    }

    fn download_output(&self, decode_output: DecodeOutput) -> Result<Vec<u8>, VulkanDecoderError> {
        let mut dst_buffer = self.copy_image_to_buffer(
            decode_output.image,
            decode_output.dimensions,
            decode_output.current_layout,
            decode_output.layer,
            &[(decode_output.wait_semaphore, vk::PipelineStageFlags2::COPY)],
            &[],
            Some(*self.sync_structures.fence_transfer_done),
        )?;

        self.sync_structures
            .fence_transfer_done
            .wait_and_reset(u64::MAX)?;

        let output = unsafe {
            self.download_data_from_buffer(
                &mut dst_buffer,
                decode_output.dimensions.width as usize
                    * decode_output.dimensions.height as usize
                    * 3
                    / 2,
            )?
        };

        Ok(output)
    }

    fn prepare_references_std_ref_info(
        decode_information: &DecodeInformation,
    ) -> Vec<vk::native::StdVideoDecodeH264ReferenceInfo> {
        decode_information
            .reference_list
            .iter()
            .flatten()
            .map(|ref_info| ref_info.picture_info.into())
            .collect::<Vec<_>>()
    }

    fn prepare_references_dpb_slot_info(
        references_std_ref_info: &[vk::native::StdVideoDecodeH264ReferenceInfo],
    ) -> Vec<vk::VideoDecodeH264DpbSlotInfoKHR> {
        references_std_ref_info
            .iter()
            .map(|info| vk::VideoDecodeH264DpbSlotInfoKHR::default().std_reference_info(info))
            .collect::<Vec<_>>()
    }

    fn prepare_reference_list_slot_info<'a>(
        reference_id_to_dpb_slot_index: &std::collections::HashMap<ReferenceId, usize>,
        reference_slots: &'a [vk::VideoReferenceSlotInfoKHR<'a>],
        references_dpb_slot_info: &'a mut [vk::VideoDecodeH264DpbSlotInfoKHR<'a>],
        decode_information: &'a DecodeInformation,
    ) -> Result<Vec<vk::VideoReferenceSlotInfoKHR<'a>>, VulkanDecoderError> {
        let mut pic_reference_slots = Vec::new();
        for (ref_info, dpb_slot_info) in decode_information
            .reference_list
            .iter()
            .flatten()
            .zip(references_dpb_slot_info.iter_mut())
        {
            let i = *reference_id_to_dpb_slot_index
                .get(&ref_info.id)
                .ok_or(VulkanDecoderError::NonExistantReferenceRequested)?;

            let reference = *reference_slots
                .get(i)
                .ok_or(VulkanDecoderError::NonExistantReferenceRequested)?;

            if reference.slot_index < 0 || reference.p_picture_resource.is_null() {
                return Err(VulkanDecoderError::NonExistantReferenceRequested);
            }

            let reference = reference.push_next(dpb_slot_info);

            pic_reference_slots.push(reference);
        }

        Ok(pic_reference_slots)
    }

    /// ## Safety
    /// the buffer has to be mappable and readable
    unsafe fn download_data_from_buffer(
        &self,
        buffer: &mut Buffer,
        size: usize,
    ) -> Result<Vec<u8>, VulkanDecoderError> {
        let mut output = Vec::new();
        unsafe {
            let memory = self
                .vulkan_ctx
                .allocator
                .map_memory(&mut buffer.allocation)?;
            let memory_slice = std::slice::from_raw_parts_mut(memory, size);
            output.extend_from_slice(memory_slice);
            self.vulkan_ctx
                .allocator
                .unmap_memory(&mut buffer.allocation);
        }

        Ok(output)
    }

    fn upload_decode_data_to_buffer(
        &self,
        data: &[u8],
        buffer_size: u64,
    ) -> Result<Buffer, VulkanDecoderError> {
        let mut decode_buffer = Buffer::new_decode(
            self.vulkan_ctx.allocator.clone(),
            buffer_size,
            &H264ProfileInfo::decode_h264_yuv420(),
        )?;

        unsafe {
            let mem = self
                .vulkan_ctx
                .allocator
                .map_memory(&mut decode_buffer.allocation)?;
            let slice = std::slice::from_raw_parts_mut(mem.cast(), data.len());
            slice.copy_from_slice(data);
            self.vulkan_ctx
                .allocator
                .unmap_memory(&mut decode_buffer.allocation);
        }

        Ok(decode_buffer)
    }

    #[allow(clippy::too_many_arguments)]
    fn copy_image_to_buffer(
        &self,
        image: vk::Image,
        dimensions: vk::Extent2D,
        current_image_layout: vk::ImageLayout,
        layer: u32,
        wait_semaphores: &[(vk::Semaphore, vk::PipelineStageFlags2)],
        signal_semaphores: &[(vk::Semaphore, vk::PipelineStageFlags2)],
        fence: Option<vk::Fence>,
    ) -> Result<Buffer, VulkanDecoderError> {
        self.command_buffers.gpu_to_mem_transfer_buffer.begin()?;

        let memory_barrier = vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::NONE)
            .src_access_mask(vk::AccessFlags2::NONE)
            .dst_stage_mask(vk::PipelineStageFlags2::COPY)
            .dst_access_mask(vk::AccessFlags2::TRANSFER_READ)
            .old_layout(current_image_layout)
            .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: layer,
                layer_count: 1,
            });

        unsafe {
            self.vulkan_ctx.device.cmd_pipeline_barrier2(
                *self.command_buffers.gpu_to_mem_transfer_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&[memory_barrier]),
            )
        };

        // TODO: in this section, we shouldn't be using `max_coded_extent` and use the real frame
        // resolution
        let y_plane_size = dimensions.width as u64 * dimensions.height as u64;

        let dst_buffer = Buffer::new_transfer(
            self.vulkan_ctx.allocator.clone(),
            y_plane_size * 3 / 2,
            TransferDirection::GpuToMem,
        )?;

        let copy_info = [
            vk::BufferImageCopy::default()
                .image_subresource(vk::ImageSubresourceLayers {
                    mip_level: 0,
                    layer_count: 1,
                    base_array_layer: layer,
                    aspect_mask: vk::ImageAspectFlags::PLANE_0,
                })
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D {
                    width: dimensions.width,
                    height: dimensions.height,
                    depth: 1,
                })
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0),
            vk::BufferImageCopy::default()
                .image_subresource(vk::ImageSubresourceLayers {
                    mip_level: 0,
                    layer_count: 1,
                    base_array_layer: layer,
                    aspect_mask: vk::ImageAspectFlags::PLANE_1,
                })
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D {
                    width: dimensions.width / 2,
                    height: dimensions.height / 2,
                    depth: 1,
                })
                .buffer_offset(y_plane_size)
                .buffer_row_length(0)
                .buffer_image_height(0),
        ];

        unsafe {
            self.vulkan_ctx.device.cmd_copy_image_to_buffer(
                *self.command_buffers.gpu_to_mem_transfer_buffer,
                image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                *dst_buffer,
                &copy_info,
            )
        };

        let memory_barrier = memory_barrier
            .src_stage_mask(vk::PipelineStageFlags2::COPY)
            .src_access_mask(vk::AccessFlags2::TRANSFER_READ)
            .dst_stage_mask(vk::PipelineStageFlags2::NONE)
            .dst_access_mask(vk::AccessFlags2::NONE)
            .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .new_layout(current_image_layout);

        unsafe {
            self.vulkan_ctx.device.cmd_pipeline_barrier2(
                *self.command_buffers.gpu_to_mem_transfer_buffer,
                &vk::DependencyInfo::default().image_memory_barriers(&[memory_barrier]),
            )
        };

        self.command_buffers.gpu_to_mem_transfer_buffer.end()?;

        self.command_buffers.gpu_to_mem_transfer_buffer.submit(
            *self.vulkan_ctx.queues.transfer.queue.lock().unwrap(),
            wait_semaphores,
            signal_semaphores,
            fence,
        )?;

        Ok(dst_buffer)
    }
}

impl From<crate::parser::PictureInfo> for vk::native::StdVideoDecodeH264ReferenceInfo {
    fn from(picture_info: crate::parser::PictureInfo) -> Self {
        vk::native::StdVideoDecodeH264ReferenceInfo {
            flags: vk::native::StdVideoDecodeH264ReferenceInfoFlags {
                __bindgen_padding_0: [0; 3],
                _bitfield_align_1: [],
                _bitfield_1: vk::native::StdVideoDecodeH264ReferenceInfoFlags::new_bitfield_1(
                    0,
                    0,
                    picture_info.used_for_long_term_reference.into(),
                    picture_info.non_existing.into(),
                ),
            },
            FrameNum: picture_info.FrameNum,
            PicOrderCnt: picture_info.PicOrderCnt,
            reserved: 0,
        }
    }
}

pub(crate) struct DecodingImages<'a> {
    pub(crate) dpb_image: DecodingImageBundle<'a>,
    pub(crate) dpb_slot_active: Vec<bool>,
    pub(crate) dst_image: Option<DecodingImageBundle<'a>>,
}

pub(crate) struct DecodingImageBundle<'a> {
    pub(crate) image: Arc<Image>,
    pub(crate) _image_view: ImageView,
    pub(crate) video_resource_info: Vec<vk::VideoPictureResourceInfoKHR<'a>>,
}

impl<'a> DecodingImageBundle<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        vulkan_ctx: &VulkanCtx,
        format: &vk::VideoFormatPropertiesKHR<'a>,
        dimensions: vk::Extent2D,
        image_usage: vk::ImageUsageFlags,
        profile_info: &H264ProfileInfo,
        array_layer_count: u32,
        queue_indices: Option<&[u32]>,
        layout: vk::ImageLayout,
    ) -> Result<(Self, vk::ImageMemoryBarrier2<'a>), VulkanDecoderError> {
        let mut profile_list_info = vk::VideoProfileListInfoKHR::default()
            .profiles(std::slice::from_ref(&profile_info.profile_info));

        let mut image_create_info = vk::ImageCreateInfo::default()
            .flags(format.image_create_flags)
            .image_type(format.image_type)
            .format(format.format)
            .extent(vk::Extent3D {
                width: dimensions.width,
                height: dimensions.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(array_layer_count)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(format.image_tiling)
            .usage(image_usage)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .push_next(&mut profile_list_info);

        match queue_indices {
            Some(indices) => {
                image_create_info = image_create_info
                    .sharing_mode(vk::SharingMode::CONCURRENT)
                    .queue_family_indices(indices);
            }
            None => {
                image_create_info = image_create_info.sharing_mode(vk::SharingMode::EXCLUSIVE);
            }
        }

        let image = Arc::new(Image::new(
            vulkan_ctx.allocator.clone(),
            &image_create_info,
        )?);

        let subresource_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: vk::REMAINING_ARRAY_LAYERS,
        };

        let image_view_create_info = vk::ImageViewCreateInfo::default()
            .flags(vk::ImageViewCreateFlags::empty())
            .image(**image)
            .view_type(if array_layer_count == 1 {
                vk::ImageViewType::TYPE_2D
            } else {
                vk::ImageViewType::TYPE_2D_ARRAY
            })
            .format(format.format)
            .components(vk::ComponentMapping::default())
            .subresource_range(subresource_range);

        let image_view = ImageView::new(
            vulkan_ctx.device.clone(),
            image.clone(),
            &image_view_create_info,
        )?;

        let video_resource_info = (0..array_layer_count)
            .map(|i| {
                vk::VideoPictureResourceInfoKHR::default()
                    .coded_offset(vk::Offset2D { x: 0, y: 0 })
                    .coded_extent(dimensions)
                    .base_array_layer(i)
                    .image_view_binding(image_view.view)
            })
            .collect();

        let image_memory_barrier = vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::NONE)
            .src_access_mask(vk::AccessFlags2::NONE)
            .dst_stage_mask(vk::PipelineStageFlags2::NONE)
            .dst_access_mask(vk::AccessFlags2::NONE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(**image)
            .subresource_range(subresource_range);

        Ok((
            Self {
                image,
                _image_view: image_view,
                video_resource_info,
            },
            image_memory_barrier,
        ))
    }
}

impl<'a> DecodingImages<'a> {
    pub(crate) fn new(
        vulkan_ctx: &VulkanCtx,
        profile: H264ProfileInfo,
        dpb_format: &vk::VideoFormatPropertiesKHR<'a>,
        dst_format: &Option<vk::VideoFormatPropertiesKHR<'a>>,
        dimensions: vk::Extent2D,
        max_dpb_slots: u32,
    ) -> Result<(Self, Vec<vk::ImageMemoryBarrier2<'a>>), VulkanDecoderError> {
        let dpb_image_usage = if dst_format.is_some() {
            dpb_format.image_usage_flags & vk::ImageUsageFlags::VIDEO_DECODE_DPB_KHR
        } else {
            dpb_format.image_usage_flags
                & (vk::ImageUsageFlags::VIDEO_DECODE_DPB_KHR
                    | vk::ImageUsageFlags::VIDEO_DECODE_DST_KHR
                    | vk::ImageUsageFlags::TRANSFER_SRC)
        };

        let queue_indices = [
            vulkan_ctx.queues.transfer.idx as u32,
            vulkan_ctx.queues.h264_decode.idx as u32,
        ];

        let (dpb_image, dpb_memory_barrier) = DecodingImageBundle::new(
            vulkan_ctx,
            dpb_format,
            dimensions,
            dpb_image_usage,
            &profile,
            max_dpb_slots,
            if dst_format.is_some() {
                None
            } else {
                Some(&queue_indices)
            },
            vk::ImageLayout::VIDEO_DECODE_DPB_KHR,
        )?;

        let output = dst_format
            .map(|dst_format| {
                let dst_image_usage = dst_format.image_usage_flags
                    & (vk::ImageUsageFlags::VIDEO_DECODE_DST_KHR
                        | vk::ImageUsageFlags::TRANSFER_SRC);
                DecodingImageBundle::new(
                    vulkan_ctx,
                    &dst_format,
                    dimensions,
                    dst_image_usage,
                    &profile,
                    1,
                    Some(&queue_indices),
                    vk::ImageLayout::VIDEO_DECODE_DST_KHR,
                )
            })
            .transpose()?;

        let (dst_image, dst_memory_barrier) = match output {
            Some((output_images, output_memory_barrier)) => {
                (Some(output_images), Some(output_memory_barrier))
            }
            None => (None, None),
        };

        let barriers = [dpb_memory_barrier]
            .into_iter()
            .chain(dst_memory_barrier)
            .collect::<Vec<_>>();

        Ok((
            Self {
                dpb_image,
                dpb_slot_active: vec![false; max_dpb_slots as usize],
                dst_image,
            },
            barriers,
        ))
    }

    fn reference_slot_info(&self) -> Vec<vk::VideoReferenceSlotInfoKHR> {
        self.dpb_image
            .video_resource_info
            .iter()
            .enumerate()
            .map(|(i, info)| {
                vk::VideoReferenceSlotInfoKHR::default()
                    .picture_resource(info)
                    .slot_index(if self.dpb_slot_active[i] {
                        i as i32
                    } else {
                        -1
                    })
            })
            .collect()
    }

    fn allocate_reference_picture(&mut self) -> Result<usize, VulkanDecoderError> {
        let i = self
            .dpb_slot_active
            .iter()
            .enumerate()
            .find(|(_, &v)| !v)
            .map(|(i, _)| i)
            .ok_or(VulkanDecoderError::NoFreeSlotsInDpb)?;

        self.dpb_slot_active[i] = true;

        Ok(i)
    }

    fn video_resource_info(&self, i: usize) -> Option<&vk::VideoPictureResourceInfoKHR> {
        self.dpb_image.video_resource_info.get(i)
    }

    fn free_reference_picture(&mut self, i: usize) -> Result<(), VulkanDecoderError> {
        self.dpb_slot_active[i] = false;

        Ok(())
    }

    fn reset_all_allocations(&mut self) {
        self.dpb_slot_active
            .iter_mut()
            .for_each(|slot| *slot = false);
    }
}

pub(crate) struct H264ProfileInfo<'a> {
    profile_info: vk::VideoProfileInfoKHR<'a>,
    h264_info_ptr: *mut vk::VideoDecodeH264ProfileInfoKHR<'a>,
}

impl H264ProfileInfo<'_> {
    fn decode_h264_yuv420() -> Self {
        let h264_profile_info = Box::leak(Box::new(
            vk::VideoDecodeH264ProfileInfoKHR::default()
                .std_profile_idc(
                    vk::native::StdVideoH264ProfileIdc_STD_VIDEO_H264_PROFILE_IDC_BASELINE,
                )
                .picture_layout(vk::VideoDecodeH264PictureLayoutFlagsKHR::PROGRESSIVE),
        ));

        let h264_info_ptr = h264_profile_info as *mut _;
        let profile_info = vk::VideoProfileInfoKHR::default()
            .video_codec_operation(vk::VideoCodecOperationFlagsKHR::DECODE_H264)
            .chroma_subsampling(vk::VideoChromaSubsamplingFlagsKHR::TYPE_420)
            .luma_bit_depth(vk::VideoComponentBitDepthFlagsKHR::TYPE_8)
            .chroma_bit_depth(vk::VideoComponentBitDepthFlagsKHR::TYPE_8)
            .push_next(h264_profile_info);

        Self {
            profile_info,
            h264_info_ptr,
        }
    }
}

impl<'a> Drop for H264ProfileInfo<'a> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.h264_info_ptr);
        }
    }
}
