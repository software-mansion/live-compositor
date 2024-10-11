use crate::error::InitPipelineError;
use std::sync::Arc;

#[derive(Debug)]
pub struct GraphicsContext {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,

    #[cfg(feature = "vk-video")]
    pub vulkan_ctx: Option<Arc<vk_video::VulkanCtx>>,
}

impl GraphicsContext {
    #[cfg(feature = "vk-video")]
    pub fn new(
        force_gpu: bool,
        features: wgpu::Features,
        limits: wgpu::Limits,
    ) -> Result<Self, InitPipelineError> {
        use compositor_render::{
            create_wgpu_ctx, error::InitRendererEngineError, required_wgpu_features,
            set_required_wgpu_limits,
        };
        use tracing::warn;

        let vulkan_features =
            features | required_wgpu_features() | wgpu::Features::TEXTURE_FORMAT_NV12;

        let limits = set_required_wgpu_limits(limits);

        match vk_video::VulkanCtx::new(vulkan_features, limits.clone()) {
            Ok(ctx) => Ok(GraphicsContext {
                device: ctx.wgpu_ctx.device.clone(),
                queue: ctx.wgpu_ctx.queue.clone(),
                vulkan_ctx: Some(ctx.into()),
            }),

            Err(err) => {
                warn!("Cannot initialize vulkan video decoding context. Reason: {err}. Initializing without vulkan video support.");

                let (device, queue) = create_wgpu_ctx(force_gpu, features, limits)
                    .map_err(InitRendererEngineError::FailedToInitWgpuCtx)?;

                Ok(GraphicsContext {
                    device,
                    queue,
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
    ) -> Result<Self, InitPipelineError> {
        let (device, queue) = create_wgpu_ctx(force_gpu, features, limits)
            .map_err(InitRendererEngineError::FailedToInitWgpuCtx)?;

        Ok(GraphicsContext { device, queue })
    }
}
