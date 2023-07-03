pub struct Renderer {
    #[allow(dead_code)]
    wgpu_ctx: WgpuCtx,
}

#[derive(Debug, thiserror::Error)]
pub enum RendererNewError {
    #[error("failed to initialize a wgpu context")]
    FailedToInitWgpuCtx(#[from] WgpuCtxNewError),
}

impl Renderer {
    pub fn new() -> Result<Self, RendererNewError> {
        Ok(Self {
            wgpu_ctx: WgpuCtx::new()?,
        })
    }
}

struct WgpuCtx {
    #[allow(dead_code)]
    device: wgpu::Device,

    #[allow(dead_code)]
    queue: wgpu::Queue,
}

#[derive(Debug, thiserror::Error)]
pub enum WgpuCtxNewError {
    #[error("failed to get a wgpu adapter")]
    NoAdapter,

    #[error("failed to get a wgpu device")]
    NoDevice(#[from] wgpu::RequestDeviceError),
}

impl WgpuCtx {
    fn new() -> Result<Self, WgpuCtxNewError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            }))
            .ok_or(WgpuCtxNewError::NoAdapter)?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Video Compositor's GPU :^)"),
                limits: Default::default(),
                features: wgpu::Features::empty(),
            },
            None,
        ))?;

        Ok(Self { device, queue })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn initialize() -> Result<(), RendererNewError> {
        Renderer::new()?;
        Ok(())
    }
}
