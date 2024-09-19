use std::{collections::HashMap, sync::Arc};

use ash::vk;
use h264_reader::nal::{pps::PicParameterSet, sps::SeqParameterSet};

use crate::{
    vulkan_decoder::{
        parameter_sets::{VkPictureParameterSet, VkSequenceParameterSet},
        VulkanDecoderError,
    },
    VulkanCtx,
};

use super::{Device, MemoryAllocation, VideoQueueExt};

/// Since `VideoSessionParameters` can only add sps and pps values (inserting sps or pps with an
/// existing id is prohibited), this is an abstraction which provides the capability to replace an
/// existing sps or pps.
pub(crate) struct VideoSessionParametersManager {
    pub(crate) parameters: VideoSessionParameters,
    sps: HashMap<u8, VkSequenceParameterSet>,
    pps: HashMap<(u8, u8), VkPictureParameterSet>,
    device: Arc<Device>,
    session: vk::VideoSessionKHR,
}

impl VideoSessionParametersManager {
    pub(crate) fn new(
        vulkan_ctx: &VulkanCtx,
        session: vk::VideoSessionKHR,
    ) -> Result<Self, VulkanDecoderError> {
        Ok(Self {
            parameters: VideoSessionParameters::new(
                vulkan_ctx.device.clone(),
                session,
                &[],
                &[],
                None,
            )?,
            sps: HashMap::new(),
            pps: HashMap::new(),
            device: vulkan_ctx.device.clone(),
            session,
        })
    }

    pub(crate) fn parameters(&self) -> vk::VideoSessionParametersKHR {
        self.parameters.parameters
    }

    pub(crate) fn change_session(
        &mut self,
        session: vk::VideoSessionKHR,
    ) -> Result<(), VulkanDecoderError> {
        if self.session == session {
            return Ok(());
        }
        self.session = session;

        let sps = self.sps.values().map(|sps| sps.sps).collect::<Vec<_>>();
        let pps = self.pps.values().map(|pps| pps.pps).collect::<Vec<_>>();

        self.parameters =
            VideoSessionParameters::new(self.device.clone(), session, &sps, &pps, None)?;

        Ok(())
    }

    // it is probably not optimal to insert sps and pps searately. this could be optimized, so that
    // the insertion happens lazily when the parameters are bound to a session.
    pub(crate) fn put_sps(&mut self, sps: &SeqParameterSet) -> Result<(), VulkanDecoderError> {
        let key = sps.seq_parameter_set_id.id();
        match self.sps.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                e.insert(sps.try_into()?);

                self.parameters = VideoSessionParameters::new(
                    self.device.clone(),
                    self.session,
                    &[self.sps[&key].sps],
                    &[],
                    Some(&self.parameters),
                )?
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(sps.try_into()?);

                self.parameters.add(&[self.sps[&key].sps], &[])?;
            }
        }

        Ok(())
    }

    pub(crate) fn put_pps(&mut self, pps: &PicParameterSet) -> Result<(), VulkanDecoderError> {
        let key = (pps.seq_parameter_set_id.id(), pps.pic_parameter_set_id.id());
        match self.pps.entry(key) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                e.insert(pps.try_into()?);

                self.parameters = VideoSessionParameters::new(
                    self.device.clone(),
                    self.session,
                    &[],
                    &[self.pps[&key].pps],
                    Some(&self.parameters),
                )?;
            }

            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(pps.try_into()?);

                self.parameters.add(&[], &[self.pps[&key].pps])?;
            }
        }

        Ok(())
    }
}

pub(crate) struct VideoSessionParameters {
    pub(crate) parameters: vk::VideoSessionParametersKHR,
    update_sequence_count: u32,
    device: Arc<Device>,
}

impl VideoSessionParameters {
    pub(crate) fn new(
        device: Arc<Device>,
        session: vk::VideoSessionKHR,
        initial_sps: &[vk::native::StdVideoH264SequenceParameterSet],
        initial_pps: &[vk::native::StdVideoH264PictureParameterSet],
        template: Option<&Self>,
    ) -> Result<Self, VulkanDecoderError> {
        let parameters_add_info = vk::VideoDecodeH264SessionParametersAddInfoKHR::default()
            .std_sp_ss(initial_sps)
            .std_pp_ss(initial_pps);

        let mut h264_create_info = vk::VideoDecodeH264SessionParametersCreateInfoKHR::default()
            .max_std_sps_count(32)
            .max_std_pps_count(32)
            .parameters_add_info(&parameters_add_info);

        let create_info = vk::VideoSessionParametersCreateInfoKHR::default()
            .flags(vk::VideoSessionParametersCreateFlagsKHR::empty())
            .video_session_parameters_template(
                template
                    .map(|t| t.parameters)
                    .unwrap_or_else(vk::VideoSessionParametersKHR::null),
            )
            .video_session(session)
            .push_next(&mut h264_create_info);

        let parameters = unsafe {
            device
                .video_queue_ext
                .create_video_session_parameters_khr(&create_info, None)?
        };

        Ok(Self {
            parameters,
            update_sequence_count: 0,
            device: device.clone(),
        })
    }

    pub(crate) fn add(
        &mut self,
        sps: &[vk::native::StdVideoH264SequenceParameterSet],
        pps: &[vk::native::StdVideoH264PictureParameterSet],
    ) -> Result<(), VulkanDecoderError> {
        let mut parameters_add_info = vk::VideoDecodeH264SessionParametersAddInfoKHR::default()
            .std_sp_ss(sps)
            .std_pp_ss(pps);

        self.update_sequence_count += 1;
        let update_info = vk::VideoSessionParametersUpdateInfoKHR::default()
            .update_sequence_count(self.update_sequence_count)
            .push_next(&mut parameters_add_info);

        unsafe {
            self.device
                .video_queue_ext
                .update_video_session_parameters_khr(self.parameters, &update_info)?
        };

        Ok(())
    }
}

impl Drop for VideoSessionParameters {
    fn drop(&mut self) {
        unsafe {
            self.device
                .video_queue_ext
                .destroy_video_session_parameters_khr(self.parameters, None)
        }
    }
}

pub(crate) struct VideoSession {
    pub(crate) session: vk::VideoSessionKHR,
    pub(crate) device: Arc<Device>,
    pub(crate) _allocations: Vec<MemoryAllocation>,
    pub(crate) max_coded_extent: vk::Extent2D,
    pub(crate) max_dpb_slots: u32,
}

impl VideoSession {
    pub(crate) fn new(
        vulkan_ctx: &VulkanCtx,
        profile_info: &vk::VideoProfileInfoKHR,
        max_coded_extent: vk::Extent2D,
        max_dpb_slots: u32,
        max_active_references: u32,
        std_header_version: &vk::ExtensionProperties,
    ) -> Result<Self, VulkanDecoderError> {
        // TODO: this probably works, but this format needs to be detected and set
        // based on what the GPU supports
        let format = vk::Format::G8_B8R8_2PLANE_420_UNORM;

        let session_create_info = vk::VideoSessionCreateInfoKHR::default()
            .queue_family_index(vulkan_ctx.queues.h264_decode.idx as u32)
            .video_profile(profile_info)
            .picture_format(format)
            .max_coded_extent(max_coded_extent)
            .reference_picture_format(format)
            .max_dpb_slots(max_dpb_slots)
            .max_active_reference_pictures(max_active_references)
            .std_header_version(std_header_version);

        let video_session = unsafe {
            vulkan_ctx
                .device
                .video_queue_ext
                .create_video_session_khr(&session_create_info, None)?
        };

        let memory_requirements = unsafe {
            vulkan_ctx
                .device
                .video_queue_ext
                .get_video_session_memory_requirements_khr(video_session)?
        };

        let allocations = memory_requirements
            .iter()
            .map(|req| {
                MemoryAllocation::new(
                    vulkan_ctx.allocator.clone(),
                    &req.memory_requirements,
                    &vk_mem::AllocationCreateInfo {
                        usage: vk_mem::MemoryUsage::Unknown,
                        ..Default::default()
                    },
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let memory_bind_infos = memory_requirements
            .into_iter()
            .zip(allocations.iter())
            .map(|(req, allocation)| {
                let allocation_info = allocation.allocation_info();
                vk::BindVideoSessionMemoryInfoKHR::default()
                    .memory_bind_index(req.memory_bind_index)
                    .memory(allocation_info.device_memory)
                    .memory_offset(allocation_info.offset)
                    .memory_size(allocation_info.size)
            })
            .collect::<Vec<_>>();

        unsafe {
            vulkan_ctx
                .device
                .video_queue_ext
                .bind_video_session_memory_khr(video_session, &memory_bind_infos)?
        };

        Ok(VideoSession {
            session: video_session,
            _allocations: allocations,
            device: vulkan_ctx.device.clone(),
            max_coded_extent,
            max_dpb_slots,
        })
    }
}

impl Drop for VideoSession {
    fn drop(&mut self) {
        unsafe {
            self.device
                .video_queue_ext
                .destroy_video_session_khr(self.session, None)
        };
    }
}
