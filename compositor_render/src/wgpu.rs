use log::error;

pub(crate) mod common_pipeline;
mod ctx;
pub(crate) mod format;
pub(crate) mod texture;

pub use ctx::use_global_wgpu_ctx;
pub(crate) use ctx::WgpuCtx;
pub use wgpu::Features as WgpuFeatures;

#[must_use]
pub(crate) struct WgpuErrorScope;

impl WgpuErrorScope {
    pub(crate) fn push(device: &wgpu::Device) -> Self {
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);

        Self
    }

    pub(crate) fn pop(self, device: &wgpu::Device) -> Result<(), WgpuError> {
        for _ in 0..2 {
            if let Some(error) = pollster::block_on(device.pop_error_scope()) {
                return Err(error.into());
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum CreateWgpuCtxError {
    #[error("Failed to get a wgpu adapter.")]
    NoAdapter,

    #[error("Failed to get a wgpu device.")]
    NoDevice(#[from] wgpu::RequestDeviceError),

    #[error(transparent)]
    WgpuError(#[from] WgpuError),
}

#[derive(Debug, thiserror::Error, Clone)]
pub enum WgpuError {
    #[error("Wgpu validation error:\n{0}")]
    Validation(String),
    #[error("Wgpu out of memory error: {0}")]
    OutOfMemory(String),
}

/// Convert to custom error because wgpu::Error is not Send/Sync
impl From<wgpu::Error> for WgpuError {
    fn from(value: wgpu::Error) -> Self {
        match value {
            wgpu::Error::OutOfMemory { .. } => Self::OutOfMemory(value.to_string()),
            wgpu::Error::Validation { .. } => Self::Validation(value.to_string()),
        }
    }
}
