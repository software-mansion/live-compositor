use std::sync::Arc;

use ash::vk;

use crate::{Frame, VulkanCtxError, VulkanDevice};

#[derive(Debug, thiserror::Error)]
pub enum VulkanEncoderError {
    #[error("Vulkan error: {0}")]
    VkError(#[from] ash::vk::Result),

    #[error("Cannot find enough memory of the right type on the deivce")]
    NoMemory,

    #[error("The supplied textures format is {0:?}, when it should be NV12")]
    NotNV12Texture(wgpu::TextureFormat),

    #[error(transparent)]
    VulkanCtxError(#[from] VulkanCtxError),
}


pub(crate) struct VulkanEncoder {
    device: Arc<VulkanDevice>,
}

impl VulkanEncoder {
    pub fn encode(&mut self, frame: Frame<wgpu::Texture>) -> Result<Vec<u8>, VulkanEncoderError> {
        if frame.frame.format() != wgpu::TextureFormat::NV12 {
            return Err(VulkanEncoderError::NotNV12Texture(frame.frame.format()));
        }


        todo!();
    }
}


