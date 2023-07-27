use std::rc::Rc;

use compositor_common::{scene::Scene, Frame};

use crate::{
    input_frames::InputFrames,
    registry::{self, TransformationRegistry},
};

use self::transformation::Transformation;

pub mod texture;
pub mod transformation;

pub struct Renderer {
    wgpu_ctx: Rc<WgpuCtx>,
    registry: TransformationRegistry,
    scene: Option<Scene>,
}

#[derive(Debug, thiserror::Error)]
pub enum RendererNewError {
    #[error("failed to initialize a wgpu context")]
    FailedToInitWgpuCtx(#[from] WgpuCtxNewError),
}

#[derive(Debug, thiserror::Error)]
pub enum RendererRegisterTransformationError {
    #[error("failed to register a transformation in the transformation registry")]
    TransformationRegistryError(#[from] registry::RegisterError),
}

#[derive(Debug, thiserror::Error)]
pub enum RendererRenderError {
    #[error("no scene was set in the compositor")]
    NoScene,

    #[error("a frame was not provided for input with id {0}")]
    NoInput(u32),
}

impl Renderer {
    pub fn new() -> Result<Self, RendererNewError> {
        Ok(Self {
            wgpu_ctx: Rc::new(WgpuCtx::new()?),
            registry: TransformationRegistry::new(),
            scene: None,
        })
    }

    pub fn register_transformation<T: Transformation>(
        &mut self,
        provider: fn(Rc<WgpuCtx>) -> T,
    ) -> Result<(), RendererRegisterTransformationError> {
        self.registry
            .register(Box::new(provider(self.wgpu_ctx.clone())))?;

        Ok(())
    }

    /// This is very much a work in progress.
    /// For now it just takes a random frame from the input and returns it
    pub fn render(&self, inputs: InputFrames) -> Result<Frame, RendererRenderError> {
        let output_frame = inputs.frames.values().next().cloned();

        match output_frame {
            Some(frame) => {
                let frame = Frame {
                    data: frame.data.clone(),
                    resolution: frame.resolution,
                    pts: inputs.pts,
                };
                Ok(frame)
            }
            None => Err(RendererRenderError::NoInput(0)),
        }
    }

    pub fn update_scene(&mut self, scene: Scene) {
        self.scene = Some(scene);
    }
}

pub struct WgpuCtx {
    #[allow(dead_code)]
    pub device: wgpu::Device,

    #[allow(dead_code)]
    pub queue: wgpu::Queue,
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
