use std::sync::Arc;

use compositor_common::{
    scene::{InputId, OutputId, SceneSpec},
    transformation::{TransformationRegistryKey, TransformationSpec},
};
use log::error;

use crate::{
    frame_set::FrameSet,
    registry::TransformationRegistry,
    render_loop::{populate_inputs, read_outputs},
};
use crate::{
    registry::{self, RegistryType},
    render_loop::run_transforms,
    transformations::{
        shader::Shader,
        web_renderer::{
            electron::ElectronNewError, ElectronInstance, WebRenderer, WebRendererNewError,
        },
    },
};

use self::{
    color_converter_pipeline::{RGBAToYUVConverter, YUVToRGBAConverter},
    scene::{Scene, SceneUpdateError},
    texture::{RGBATexture, YUVTextures},
};

mod color_converter_pipeline;
pub mod common_pipeline;
pub mod scene;
pub mod texture;

pub struct Renderer {
    pub wgpu_ctx: Arc<WgpuCtx>,
    pub electron_instance: Arc<ElectronInstance>,
    pub scene: Scene,
    pub shader_transforms: TransformationRegistry<Arc<Shader>>,
    pub web_renderers: TransformationRegistry<Arc<WebRenderer>>,
}

pub struct RenderCtx<'a> {
    pub wgpu_ctx: &'a Arc<WgpuCtx>,
    pub electron: &'a Arc<ElectronInstance>,
    pub shader_transforms: &'a TransformationRegistry<Arc<Shader>>,
    pub web_renderers: &'a TransformationRegistry<Arc<WebRenderer>>,
}

#[derive(Debug, thiserror::Error)]
pub enum RendererNewError {
    #[error("failed to initialize a wgpu context")]
    FailedToInitWgpuCtx(#[from] WgpuCtxNewError),

    #[error("failed to start an electron instance")]
    FailedToStartElectron(#[from] ElectronNewError),
}

#[derive(Debug, thiserror::Error)]
pub enum RendererRegisterTransformationError {
    #[error("failed to register a transformation in the transformation registry")]
    TransformationRegistry(#[from] registry::RegisterError),

    #[error("failed to create web renderer transformation")]
    WebRendererTransformation(#[from] WebRendererNewError),
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
            wgpu_ctx: Arc::new(WgpuCtx::new()?),
            electron_instance: Arc::new(ElectronInstance::new(9002)?), // TODO: make it configurable
            scene: Scene::empty(),
            web_renderers: TransformationRegistry::new(RegistryType::WebRenderer),
            shader_transforms: TransformationRegistry::new(RegistryType::Shader),
        })
    }

    fn ctx(&self) -> RenderCtx {
        RenderCtx {
            wgpu_ctx: &self.wgpu_ctx,
            electron: &self.electron_instance,
            shader_transforms: &self.shader_transforms,
            web_renderers: &self.web_renderers,
        }
    }

    pub fn register_transformation(
        &mut self,
        key: TransformationRegistryKey,
        spec: TransformationSpec,
    ) -> Result<(), RendererRegisterTransformationError> {
        match spec {
            TransformationSpec::Shader { source } => self
                .shader_transforms
                .register(&key, Arc::new(Shader::new(&self.ctx(), source)))?,
            TransformationSpec::WebRenderer(params) => self
                .web_renderers
                .register(&key, Arc::new(WebRenderer::new(&self.ctx(), params)?))?,
        };
        Ok(())
    }

    pub fn render(
        &self,
        mut inputs: FrameSet<InputId>,
    ) -> Result<FrameSet<OutputId>, RendererRenderError> {
        let ctx = self.ctx();

        populate_inputs(&ctx, &self.scene, &mut inputs.frames);
        run_transforms(&ctx, &self.scene);
        let frames = read_outputs(&ctx, &self.scene, inputs.pts);

        Ok(FrameSet {
            frames,
            pts: inputs.pts,
        })
    }

    pub fn update_scene(&mut self, scene_specs: SceneSpec) -> Result<(), SceneUpdateError> {
        self.scene.update(
            &RenderCtx {
                wgpu_ctx: &self.wgpu_ctx,
                electron: &self.electron_instance,
                shader_transforms: &self.shader_transforms,
                web_renderers: &self.web_renderers,
            },
            scene_specs,
        )
    }
}

pub struct WgpuCtx {
    #[allow(dead_code)]
    pub device: wgpu::Device,

    #[allow(dead_code)]
    pub queue: wgpu::Queue,

    pub yuv_bind_group_layout: wgpu::BindGroupLayout,
    pub rgba_bind_group_layout: wgpu::BindGroupLayout,
    pub yuv_to_rgba_converter: YUVToRGBAConverter,
    pub rgba_to_yuv_converter: RGBAToYUVConverter,
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

        let yuv_bind_group_layout = YUVTextures::new_bind_group_layout(&device);
        let rgba_bind_group_layout = RGBATexture::new_bind_group_layout(&device);
        let yuv_to_rgba_converter = YUVToRGBAConverter::new(&device, &yuv_bind_group_layout);
        let rgba_to_yuv_converter = RGBAToYUVConverter::new(&device, &rgba_bind_group_layout);

        Ok(Self {
            device,
            queue,
            yuv_bind_group_layout,
            rgba_bind_group_layout,
            yuv_to_rgba_converter,
            rgba_to_yuv_converter,
        })
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
