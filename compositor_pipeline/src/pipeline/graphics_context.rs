use crate::error::InitPipelineError;
use compositor_render::{create_wgpu_ctx, error::InitRendererEngineError, WgpuComponents};
use std::sync::Arc;

#[cfg(feature = "vk-video")]
#[derive(Debug, Clone)]
pub struct VulkanCtx {
    pub device: Arc<vk_video::VulkanDevice>,
    pub instance: Arc<vk_video::VulkanInstance>,
}

#[derive(Debug, Clone)]
pub struct GraphicsContext {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub adapter: Arc<wgpu::Adapter>,
    pub instance: Arc<wgpu::Instance>,

    #[cfg(feature = "vk-video")]
    pub vulkan_ctx: Option<VulkanCtx>,
}

impl GraphicsContext {
    #[cfg(feature = "vk-video")]
    pub fn new(
        force_gpu: bool,
        features: wgpu::Features,
        limits: wgpu::Limits,
        mut compatible_surface: Option<&mut wgpu::Surface<'_>>,
    ) -> Result<Self, InitPipelineError> {
        use compositor_render::{required_wgpu_features, set_required_wgpu_limits};
        use tracing::warn;

        let vulkan_features =
            features | required_wgpu_features() | wgpu::Features::TEXTURE_FORMAT_NV12;

        let limits = set_required_wgpu_limits(limits);

        match vk_video::VulkanInstance::new().and_then(|instance| {
            let device =
                instance.create_device(vulkan_features, limits.clone(), &mut compatible_surface)?;

            Ok((instance, device))
        }) {
            Ok((instance, device)) => Ok(GraphicsContext {
                device: device.wgpu_device.clone(),
                queue: device.wgpu_queue.clone(),
                adapter: device.wgpu_adapter.clone(),
                instance: instance.wgpu_instance.clone(),
                vulkan_ctx: Some(VulkanCtx { instance, device }),
            }),

            Err(err) => {
                warn!("Cannot initialize vulkan video decoding context. Reason: {err}. Initializing without vulkan video support.");

                let WgpuComponents {
                    instance,
                    adapter,
                    device,
                    queue,
                } = create_wgpu_ctx(force_gpu, features, limits, compatible_surface.as_deref())
                    .map_err(InitRendererEngineError::FailedToInitWgpuCtx)?;

                Ok(GraphicsContext {
                    device,
                    queue,
                    adapter,
                    instance,
                    vulkan_ctx: None,
                })
            }
        }
    }

    #[cfg(not(feature = "vk-video"))]
    pub fn new(
        force_gpu: bool,
        features: wgpu::Features,
        limits: wgpu::Limits,
        compatible_surface: Option<&mut wgpu::Surface<'_>>,
    ) -> Result<Self, InitPipelineError> {
        let WgpuComponents {
            instance,
            adapter,
            device,
            queue,
        } = create_wgpu_ctx(force_gpu, features, limits, compatible_surface.as_deref())
            .map_err(InitRendererEngineError::FailedToInitWgpuCtx)?;

        Ok(GraphicsContext {
            device,
            queue,
            adapter,
            instance,
        })
    }
}
