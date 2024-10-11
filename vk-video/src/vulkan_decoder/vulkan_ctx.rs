use std::{
    ffi::{c_void, CStr},
    sync::Arc,
};

use ash::{vk, Entry};
use tracing::{error, info};

use super::{
    Allocator, CommandBuffer, CommandPool, DebugMessenger, Device, H264ProfileInfo, Instance,
    VulkanDecoderError,
};

const REQUIRED_EXTENSIONS: &[&CStr] = &[
    vk::KHR_VIDEO_QUEUE_NAME,
    vk::KHR_VIDEO_DECODE_QUEUE_NAME,
    vk::KHR_VIDEO_DECODE_H264_NAME,
];

#[derive(thiserror::Error, Debug)]
pub enum VulkanCtxError {
    #[error("Error loading vulkan: {0}")]
    LoadingError(#[from] ash::LoadingError),

    #[error("Vulkan error: {0}")]
    VkError(#[from] vk::Result),

    #[error("wgpu instance error: {0}")]
    WgpuInstanceError(#[from] wgpu::hal::InstanceError),

    #[error("wgpu device error: {0}")]
    WgpuDeviceError(#[from] wgpu::hal::DeviceError),

    #[error("wgpu request device error: {0}")]
    WgpuRequestDeviceError(#[from] wgpu::RequestDeviceError),

    #[error("cannot create a wgpu adapter")]
    WgpuAdapterNotCreated,

    #[error("Cannot find a suitable physical device")]
    NoDevice,

    #[error("String conversion error: {0}")]
    StringConversionError(#[from] std::ffi::FromBytesUntilNulError),
}

pub struct VulkanCtx {
    _entry: Arc<Entry>,
    _instance: Arc<Instance>,
    _physical_device: vk::PhysicalDevice,
    pub(crate) device: Arc<Device>,
    pub(crate) allocator: Arc<Allocator>,
    pub(crate) queues: Queues,
    _debug_messenger: Option<DebugMessenger>,
    pub(crate) video_capabilities: vk::VideoCapabilitiesKHR<'static>,
    pub(crate) h264_dpb_format_properties: vk::VideoFormatPropertiesKHR<'static>,
    pub(crate) h264_dst_format_properties: Option<vk::VideoFormatPropertiesKHR<'static>>,
    pub wgpu_ctx: WgpuCtx,
}

pub struct WgpuCtx {
    pub instance: Arc<wgpu::Instance>,
    pub adapter: Arc<wgpu::Adapter>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
}

pub(crate) struct CommandPools {
    pub(crate) _decode_pool: Arc<CommandPool>,
    pub(crate) _transfer_pool: Arc<CommandPool>,
}

pub(crate) struct Queue {
    pub(crate) queue: std::sync::Mutex<vk::Queue>,
    pub(crate) idx: usize,
    _video_properties: vk::QueueFamilyVideoPropertiesKHR<'static>,
    pub(crate) query_result_status_properties:
        vk::QueueFamilyQueryResultStatusPropertiesKHR<'static>,
    device: Arc<Device>,
}

impl Queue {
    pub(crate) fn supports_result_status_queries(&self) -> bool {
        self.query_result_status_properties
            .query_result_status_support
            == vk::TRUE
    }

    pub(crate) fn submit(
        &self,
        buffer: &CommandBuffer,
        wait_semaphores: &[(vk::Semaphore, vk::PipelineStageFlags2)],
        signal_semaphores: &[(vk::Semaphore, vk::PipelineStageFlags2)],
        fence: Option<vk::Fence>,
    ) -> Result<(), VulkanDecoderError> {
        fn to_sem_submit_info(
            submits: &[(vk::Semaphore, vk::PipelineStageFlags2)],
        ) -> Vec<vk::SemaphoreSubmitInfo> {
            submits
                .iter()
                .map(|&(sem, stage)| {
                    vk::SemaphoreSubmitInfo::default()
                        .semaphore(sem)
                        .stage_mask(stage)
                })
                .collect::<Vec<_>>()
        }

        let wait_semaphores = to_sem_submit_info(wait_semaphores);
        let signal_semaphores = to_sem_submit_info(signal_semaphores);

        let buffer_submit_info =
            [vk::CommandBufferSubmitInfo::default().command_buffer(buffer.buffer)];

        let submit_info = [vk::SubmitInfo2::default()
            .wait_semaphore_infos(&wait_semaphores)
            .signal_semaphore_infos(&signal_semaphores)
            .command_buffer_infos(&buffer_submit_info)];

        unsafe {
            self.device.queue_submit2(
                *self.queue.lock().unwrap(),
                &submit_info,
                fence.unwrap_or(vk::Fence::null()),
            )?
        };

        Ok(())
    }
}

pub(crate) struct Queues {
    pub(crate) transfer: Queue,
    pub(crate) h264_decode: Queue,
    pub(crate) wgpu: Queue,
}

impl VulkanCtx {
    pub fn new(
        wgpu_features: wgpu::Features,
        wgpu_limits: wgpu::Limits,
    ) -> Result<Self, VulkanCtxError> {
        let entry = Arc::new(unsafe { Entry::load()? });

        let instance_extension_properties =
            unsafe { entry.enumerate_instance_extension_properties(None)? };
        info!(
            "instance_extension_properties amount: {}",
            instance_extension_properties.len()
        );

        let api_version = vk::make_api_version(0, 1, 3, 0);
        let app_info = vk::ApplicationInfo {
            api_version,
            ..Default::default()
        };

        let requested_layers = if cfg!(debug_assertions) {
            vec![c"VK_LAYER_KHRONOS_validation"]
        } else {
            Vec::new()
        };

        let instance_layer_properties = unsafe { entry.enumerate_instance_layer_properties()? };
        let instance_layer_names = instance_layer_properties
            .iter()
            .map(|layer| layer.layer_name_as_c_str())
            .collect::<Result<Vec<_>, _>>()?;

        let layers = requested_layers
            .into_iter()
            .filter(|requested_layer_name| {
                instance_layer_names
                    .iter()
                    .any(|instance_layer_name| instance_layer_name == requested_layer_name)
            })
            .map(|layer| layer.as_ptr())
            .collect::<Vec<_>>();

        let extensions = if cfg!(debug_assertions) {
            vec![vk::EXT_DEBUG_UTILS_NAME]
        } else {
            Vec::new()
        };

        let wgpu_extensions = wgpu::hal::vulkan::Instance::desired_extensions(
            &entry,
            api_version,
            wgpu::InstanceFlags::empty(),
        )?;

        let extensions = extensions
            .into_iter()
            .chain(wgpu_extensions)
            .collect::<Vec<_>>();

        let extension_ptrs = extensions.iter().map(|e| e.as_ptr()).collect::<Vec<_>>();

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extension_ptrs);

        let instance = unsafe { entry.create_instance(&create_info, None) }?;
        let video_queue_instance_ext = ash::khr::video_queue::Instance::new(&entry, &instance);
        let debug_utils_instance_ext = ash::ext::debug_utils::Instance::new(&entry, &instance);

        let instance = Arc::new(Instance {
            instance,
            _entry: entry.clone(),
            video_queue_instance_ext,
            debug_utils_instance_ext,
        });

        let debug_messenger = if cfg!(debug_assertions) {
            Some(DebugMessenger::new(instance.clone())?)
        } else {
            None
        };

        let wgpu_instance = unsafe {
            wgpu::hal::vulkan::Instance::from_raw(
                (*entry).clone(),
                instance.instance.clone(),
                api_version,
                0,
                None,
                extensions,
                wgpu::InstanceFlags::empty(),
                false,
                None,
            )?
        };

        let physical_devices = unsafe { instance.enumerate_physical_devices()? };

        let ChosenDevice {
            physical_device,
            queue_indices,
            h264_dpb_format_properties,
            h264_dst_format_properties,
            video_capabilities,
        } = find_device(&physical_devices, &instance, REQUIRED_EXTENSIONS)?;

        let wgpu_adapter = wgpu_instance
            .expose_adapter(physical_device)
            .ok_or(VulkanCtxError::WgpuAdapterNotCreated)?;

        let wgpu_features = wgpu_features | wgpu::Features::TEXTURE_FORMAT_NV12;

        // TODO: we can only get the required extensions after exposing the adapter; the creation
        // of the adapter and verification of whether the device supports all extensions should
        // happen while picking the device.
        let wgpu_extensions = wgpu_adapter
            .adapter
            .required_device_extensions(wgpu_features);

        let required_extensions = REQUIRED_EXTENSIONS
            .iter()
            .copied()
            .chain(wgpu_extensions)
            .collect::<Vec<_>>();

        let required_extensions_as_ptrs = required_extensions
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        let queue_create_infos = queue_indices.queue_create_infos();

        let mut wgpu_physical_device_features = wgpu_adapter
            .adapter
            .physical_device_features(&required_extensions, wgpu_features);

        let mut vk_synch_2_feature =
            vk::PhysicalDeviceSynchronization2Features::default().synchronization2(true);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&required_extensions_as_ptrs);

        let device_create_info = wgpu_physical_device_features
            .add_to_device_create(device_create_info)
            .push_next(&mut vk_synch_2_feature);

        let device = unsafe { instance.create_device(physical_device, &device_create_info, None)? };
        let video_queue_ext = ash::khr::video_queue::Device::new(&instance, &device);
        let video_decode_queue_ext = ash::khr::video_decode_queue::Device::new(&instance, &device);

        let device = Arc::new(Device {
            device,
            video_queue_ext,
            video_decode_queue_ext,
            _instance: instance.clone(),
        });

        let h264_decode_queue =
            unsafe { device.get_device_queue(queue_indices.h264_decode.idx as u32, 0) };
        let transfer_queue =
            unsafe { device.get_device_queue(queue_indices.transfer.idx as u32, 0) };
        let wgpu_queue = unsafe {
            device.get_device_queue(queue_indices.graphics_transfer_compute.idx as u32, 0)
        };

        let queues = Queues {
            transfer: Queue {
                queue: transfer_queue.into(),
                idx: queue_indices.transfer.idx,
                _video_properties: queue_indices.transfer.video_properties,
                query_result_status_properties: queue_indices
                    .transfer
                    .query_result_status_properties,
                device: device.clone(),
            },
            h264_decode: Queue {
                queue: h264_decode_queue.into(),
                idx: queue_indices.h264_decode.idx,
                _video_properties: queue_indices.h264_decode.video_properties,
                query_result_status_properties: queue_indices
                    .h264_decode
                    .query_result_status_properties,
                device: device.clone(),
            },
            wgpu: Queue {
                queue: wgpu_queue.into(),
                idx: queue_indices.graphics_transfer_compute.idx,
                _video_properties: queue_indices.graphics_transfer_compute.video_properties,
                query_result_status_properties: queue_indices
                    .graphics_transfer_compute
                    .query_result_status_properties,
                device: device.clone(),
            },
        };

        let wgpu_device = unsafe {
            wgpu_adapter.adapter.device_from_raw(
                device.device.clone(),
                false,
                &required_extensions,
                wgpu_features,
                &wgpu::MemoryHints::default(),
                queue_indices.graphics_transfer_compute.idx as u32,
                0,
            )?
        };

        let allocator = Arc::new(Allocator::new(
            instance.clone(),
            physical_device,
            device.clone(),
        )?);

        let wgpu_instance =
            unsafe { wgpu::Instance::from_hal::<wgpu::hal::vulkan::Api>(wgpu_instance) };
        let wgpu_adapter = unsafe { wgpu_instance.create_adapter_from_hal(wgpu_adapter) };
        let (wgpu_device, wgpu_queue) = unsafe {
            wgpu_adapter.create_device_from_hal(
                wgpu_device,
                &wgpu::DeviceDescriptor {
                    label: Some("wgpu device created by the vulkan video decoder"),
                    memory_hints: wgpu::MemoryHints::default(),
                    required_limits: wgpu_limits,
                    required_features: wgpu_features,
                },
                None,
            )?
        };

        let wgpu_ctx = WgpuCtx {
            instance: Arc::new(wgpu_instance),
            adapter: Arc::new(wgpu_adapter),
            device: Arc::new(wgpu_device),
            queue: Arc::new(wgpu_queue),
        };

        Ok(Self {
            _entry: entry,
            _instance: instance,
            _physical_device: physical_device,
            device,
            allocator,
            queues,
            _debug_messenger: debug_messenger,
            video_capabilities,
            h264_dpb_format_properties,
            h264_dst_format_properties,
            wgpu_ctx,
        })
    }
}

impl std::fmt::Debug for VulkanCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VulkanCtx").finish()
    }
}

struct ChosenDevice<'a> {
    physical_device: vk::PhysicalDevice,
    queue_indices: QueueIndices<'a>,
    h264_dpb_format_properties: vk::VideoFormatPropertiesKHR<'a>,
    h264_dst_format_properties: Option<vk::VideoFormatPropertiesKHR<'a>>,
    video_capabilities: vk::VideoCapabilitiesKHR<'a>,
}

fn find_device<'a>(
    devices: &[vk::PhysicalDevice],
    instance: &Instance,
    required_extension_names: &[&CStr],
) -> Result<ChosenDevice<'a>, VulkanCtxError> {
    for &device in devices {
        let properties = unsafe { instance.get_physical_device_properties(device) };

        let mut vk_13_features = vk::PhysicalDeviceVulkan13Features::default();
        let mut features = vk::PhysicalDeviceFeatures2::default().push_next(&mut vk_13_features);

        unsafe { instance.get_physical_device_features2(device, &mut features) };
        let extensions = unsafe { instance.enumerate_device_extension_properties(device)? };

        if vk_13_features.synchronization2 == 0 {
            error!(
                "device {:?} does not support the required synchronization2 feature",
                properties.device_name_as_c_str()?
            );
        }

        if !required_extension_names.iter().all(|&extension_name| {
            extensions.iter().any(|ext| {
                let Ok(name) = ext.extension_name_as_c_str() else {
                    return false;
                };

                if name != extension_name {
                    return false;
                };

                true
            })
        }) {
            error!(
                "device {:?} does not support the required extensions",
                properties.device_name_as_c_str()?
            );
            continue;
        }

        let queues_len =
            unsafe { instance.get_physical_device_queue_family_properties2_len(device) };
        let mut queues = vec![vk::QueueFamilyProperties2::default(); queues_len];
        let mut video_properties = vec![vk::QueueFamilyVideoPropertiesKHR::default(); queues_len];
        let mut query_result_status_properties =
            vec![vk::QueueFamilyQueryResultStatusPropertiesKHR::default(); queues_len];

        for ((queue, video_properties), query_result_properties) in queues
            .iter_mut()
            .zip(video_properties.iter_mut())
            .zip(query_result_status_properties.iter_mut())
        {
            *queue = queue
                .push_next(video_properties)
                .push_next(query_result_properties);
        }

        unsafe { instance.get_physical_device_queue_family_properties2(device, &mut queues) };

        let profile_info = H264ProfileInfo::decode_h264_yuv420();

        let mut h264_caps = vk::VideoDecodeH264CapabilitiesKHR::default();
        let mut decode_caps = vk::VideoDecodeCapabilitiesKHR {
            p_next: (&mut h264_caps as *mut _) as *mut c_void, // why does this not have `.push_next()`? wtf
            ..Default::default()
        };

        let mut caps = vk::VideoCapabilitiesKHR::default().push_next(&mut decode_caps);

        unsafe {
            (instance
                .video_queue_instance_ext
                .fp()
                .get_physical_device_video_capabilities_khr)(
                device,
                &profile_info.profile_info,
                &mut caps,
            )
            .result()?
        };

        let video_capabilities = vk::VideoCapabilitiesKHR::default()
            .flags(caps.flags)
            .min_bitstream_buffer_size_alignment(caps.min_bitstream_buffer_size_alignment)
            .min_bitstream_buffer_offset_alignment(caps.min_bitstream_buffer_offset_alignment)
            .picture_access_granularity(caps.picture_access_granularity)
            .min_coded_extent(caps.min_coded_extent)
            .max_coded_extent(caps.max_coded_extent)
            .max_dpb_slots(caps.max_dpb_slots)
            .max_active_reference_pictures(caps.max_active_reference_pictures)
            .std_header_version(caps.std_header_version);
        info!("caps: {caps:#?}");

        let flags = decode_caps.flags;

        let h264_dpb_format_properties =
            if flags.contains(vk::VideoDecodeCapabilityFlagsKHR::DPB_AND_OUTPUT_COINCIDE) {
                query_video_format_properties(
                    device,
                    &instance.video_queue_instance_ext,
                    &profile_info,
                    vk::ImageUsageFlags::VIDEO_DECODE_DST_KHR
                        | vk::ImageUsageFlags::VIDEO_DECODE_DPB_KHR
                        | vk::ImageUsageFlags::TRANSFER_SRC,
                )?
            } else {
                query_video_format_properties(
                    device,
                    &instance.video_queue_instance_ext,
                    &profile_info,
                    vk::ImageUsageFlags::VIDEO_DECODE_DPB_KHR,
                )?
            };

        let h264_dst_format_properties =
            if flags.contains(vk::VideoDecodeCapabilityFlagsKHR::DPB_AND_OUTPUT_COINCIDE) {
                None
            } else {
                Some(query_video_format_properties(
                    device,
                    &instance.video_queue_instance_ext,
                    &profile_info,
                    vk::ImageUsageFlags::VIDEO_DECODE_DST_KHR | vk::ImageUsageFlags::TRANSFER_SRC,
                )?)
            };

        let h264_dpb_format_properties =
            if flags.contains(vk::VideoDecodeCapabilityFlagsKHR::DPB_AND_OUTPUT_COINCIDE) {
                match h264_dpb_format_properties
                    .into_iter()
                    .find(|f| f.format == vk::Format::G8_B8R8_2PLANE_420_UNORM)
                {
                    Some(f) => f,
                    None => continue,
                }
            } else {
                h264_dpb_format_properties[0]
            };

        let h264_dst_format_properties = match h264_dst_format_properties {
            Some(format_properties) => match format_properties
                .into_iter()
                .find(|f| f.format == vk::Format::G8_B8R8_2PLANE_420_UNORM)
            {
                Some(f) => Some(f),
                None => continue,
            },
            None => None,
        };

        let video_queues = queues
            .iter()
            .enumerate()
            .filter(|(_, q)| {
                q.queue_family_properties
                    .queue_flags
                    .contains(vk::QueueFlags::VIDEO_DECODE_KHR)
            })
            .map(|(i, _)| i)
            .collect::<Vec<_>>(); // TODO: have to split the queues

        let Some(transfer_queue_idx) = queues
            .iter()
            .enumerate()
            .find(|(_, q)| {
                q.queue_family_properties
                    .queue_flags
                    .contains(vk::QueueFlags::TRANSFER)
                    && !q
                        .queue_family_properties
                        .queue_flags
                        .intersects(vk::QueueFlags::GRAPHICS)
            })
            .map(|(i, _)| i)
        else {
            continue;
        };

        let Some(graphics_transfer_compute_queue_idx) = queues
            .iter()
            .enumerate()
            .find(|(_, q)| {
                q.queue_family_properties.queue_flags.contains(
                    vk::QueueFlags::GRAPHICS | vk::QueueFlags::TRANSFER | vk::QueueFlags::COMPUTE,
                )
            })
            .map(|(i, _)| i)
        else {
            continue;
        };

        let Some(decode_queue_idx) = video_queues.into_iter().find(|&i| {
            video_properties[i]
                .video_codec_operations
                .contains(vk::VideoCodecOperationFlagsKHR::DECODE_H264)
        }) else {
            continue;
        };

        info!("deocde_caps: {decode_caps:#?}");
        info!("h264_caps: {h264_caps:#?}");
        info!("dpb_format_properties: {h264_dpb_format_properties:#?}");
        info!("dst_format_properties: {h264_dst_format_properties:#?}");

        return Ok(ChosenDevice {
            physical_device: device,
            queue_indices: QueueIndices {
                transfer: QueueIndex {
                    idx: transfer_queue_idx,
                    video_properties: video_properties[transfer_queue_idx],
                    query_result_status_properties: query_result_status_properties
                        [transfer_queue_idx],
                },
                h264_decode: QueueIndex {
                    idx: decode_queue_idx,
                    video_properties: video_properties[decode_queue_idx],
                    query_result_status_properties: query_result_status_properties
                        [decode_queue_idx],
                },
                graphics_transfer_compute: QueueIndex {
                    idx: graphics_transfer_compute_queue_idx,
                    video_properties: video_properties[graphics_transfer_compute_queue_idx],
                    query_result_status_properties: query_result_status_properties
                        [graphics_transfer_compute_queue_idx],
                },
            },
            h264_dpb_format_properties,
            h264_dst_format_properties,
            video_capabilities,
        });
    }

    Err(VulkanCtxError::NoDevice)
}

fn query_video_format_properties<'a>(
    device: vk::PhysicalDevice,
    video_queue_instance_ext: &ash::khr::video_queue::Instance,
    profile_info: &H264ProfileInfo,
    image_usage: vk::ImageUsageFlags,
) -> Result<Vec<vk::VideoFormatPropertiesKHR<'a>>, VulkanCtxError> {
    let mut profile_list_info = vk::VideoProfileListInfoKHR::default()
        .profiles(std::slice::from_ref(&profile_info.profile_info));

    let format_info = vk::PhysicalDeviceVideoFormatInfoKHR::default()
        .image_usage(image_usage)
        .push_next(&mut profile_list_info);

    let mut format_info_length = 0;

    unsafe {
        (video_queue_instance_ext
            .fp()
            .get_physical_device_video_format_properties_khr)(
            device,
            &format_info,
            &mut format_info_length,
            std::ptr::null_mut(),
        )
        .result()?;
    }

    let mut format_properties =
        vec![vk::VideoFormatPropertiesKHR::default(); format_info_length as usize];

    unsafe {
        (video_queue_instance_ext
            .fp()
            .get_physical_device_video_format_properties_khr)(
            device,
            &format_info,
            &mut format_info_length,
            format_properties.as_mut_ptr(),
        )
        .result()?;
    }

    Ok(format_properties)
}

struct QueueIndex<'a> {
    idx: usize,
    video_properties: vk::QueueFamilyVideoPropertiesKHR<'a>,
    query_result_status_properties: vk::QueueFamilyQueryResultStatusPropertiesKHR<'a>,
}

pub(crate) struct QueueIndices<'a> {
    transfer: QueueIndex<'a>,
    h264_decode: QueueIndex<'a>,
    graphics_transfer_compute: QueueIndex<'a>,
}

impl QueueIndices<'_> {
    fn queue_create_infos(&self) -> Vec<vk::DeviceQueueCreateInfo> {
        [
            self.h264_decode.idx,
            self.transfer.idx,
            self.graphics_transfer_compute.idx,
        ]
        .into_iter()
        .collect::<std::collections::HashSet<usize>>()
        .into_iter()
        .map(|i| {
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(i as u32)
                .queue_priorities(&[1.0])
        })
        .collect::<Vec<_>>()
    }
}
