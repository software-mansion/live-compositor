use std::{sync::Arc, time::Duration};

use compositor_common::{
    scene::{InputId, OutputId, SceneSpec},
    Framerate,
};
use log::error;

use crate::{
    frame_set::FrameSet,
    registry::TransformationRegistry,
    render_loop::{populate_inputs, read_outputs},
    transformations::{
        shader,
        text_renderer::TextRendererCtx,
        web_renderer::chromium::{ChromiumContext, ChromiumContextError},
    },
    WebRendererOptions,
};
use crate::{
    registry::{self, RegistryType},
    render_loop::run_transforms,
    transformations::{
        shader::Shader,
        web_renderer::{WebRenderer, WebRendererNewError},
    },
};

use self::{
    color_converter_pipeline::{BGRAToRGBAConverter, RGBAToYUVConverter, YUVToRGBAConverter},
    scene::{Scene, SceneUpdateError},
    texture::{BGRATexture, RGBATexture, YUVTextures},
};

mod color_converter_pipeline;
pub mod common_pipeline;
pub mod scene;
pub mod texture;

pub struct Renderer {
    pub wgpu_ctx: Arc<WgpuCtx>,
    pub text_renderer_ctx: TextRendererCtx,
    pub chromium_context: Arc<ChromiumContext>,
    pub scene: Scene,
    pub scene_spec: Arc<SceneSpec>,
    pub(crate) shader_transforms: TransformationRegistry<Arc<Shader>>,
    pub(crate) web_renderers: TransformationRegistry<Arc<WebRenderer>>,
}

pub struct RenderCtx<'a> {
    pub wgpu_ctx: &'a Arc<WgpuCtx>,
    pub text_renderer_ctx: &'a TextRendererCtx,
    pub chromium: &'a Arc<ChromiumContext>,
    pub(crate) shader_transforms: &'a TransformationRegistry<Arc<Shader>>,
    pub(crate) web_renderers: &'a TransformationRegistry<Arc<WebRenderer>>,
}

pub struct RegisterTransformationCtx {
    pub wgpu_ctx: Arc<WgpuCtx>,
    pub chromium: Arc<ChromiumContext>,
}

#[derive(Debug, thiserror::Error)]
pub enum RendererNewError {
    #[error("failed to initialize a wgpu context")]
    FailedToInitWgpuCtx(#[from] WgpuCtxNewError),

    #[error("failed to init chromium context")]
    FailedToStartElectron(#[from] ChromiumContextError),
}

#[derive(Debug, thiserror::Error)]
pub enum RendererRegisterTransformationError {
    #[error("failed to register a transformation in the transformation registry")]
    TransformationRegistry(#[from] registry::RegisterError),

    #[error("failed to create web renderer transformation")]
    WebRendererTransformation(#[from] WebRendererNewError),
}

impl Renderer {
    pub fn new(
        web_renderer_opts: WebRendererOptions,
        framerate: Framerate,
    ) -> Result<Self, RendererNewError> {
        Ok(Self {
            wgpu_ctx: Arc::new(WgpuCtx::new()?),
            text_renderer_ctx: TextRendererCtx::new(),
            chromium_context: Arc::new(ChromiumContext::new(web_renderer_opts, framerate)?),
            scene: Scene::empty(),
            web_renderers: TransformationRegistry::new(RegistryType::WebRenderer),
            shader_transforms: TransformationRegistry::new(RegistryType::Shader),
            scene_spec: Arc::new(SceneSpec {
                inputs: vec![],
                transforms: vec![],
                outputs: vec![],
            }),
        })
    }

    pub(super) fn register_transformation_ctx(&self) -> RegisterTransformationCtx {
        RegisterTransformationCtx {
            wgpu_ctx: self.wgpu_ctx.clone(),
            chromium: self.chromium_context.clone(),
        }
    }

    pub fn render(&mut self, mut inputs: FrameSet<InputId>) -> FrameSet<OutputId> {
        let ctx = &mut RenderCtx {
            wgpu_ctx: &self.wgpu_ctx,
            chromium: &self.chromium_context,
            shader_transforms: &self.shader_transforms,
            web_renderers: &self.web_renderers,
            text_renderer_ctx: &self.text_renderer_ctx,
        };

        shader::prepare_render_loop(ctx, inputs.pts);
        populate_inputs(ctx, &mut self.scene, &mut inputs.frames);
        run_transforms(ctx, &self.scene);
        let frames = read_outputs(ctx, &self.scene, inputs.pts);

        FrameSet {
            frames,
            pts: inputs.pts,
        }
    }

    pub fn update_scene(&mut self, scene_specs: Arc<SceneSpec>) -> Result<(), SceneUpdateError> {
        self.scene.update(
            &RenderCtx {
                wgpu_ctx: &self.wgpu_ctx,
                text_renderer_ctx: &self.text_renderer_ctx,
                chromium: &self.chromium_context,
                shader_transforms: &self.shader_transforms,
                web_renderers: &self.web_renderers,
            },
            &scene_specs,
        )?;
        self.scene_spec = scene_specs;
        Ok(())
    }
}

pub struct WgpuCtx {
    #[allow(dead_code)]
    pub device: wgpu::Device,

    #[allow(dead_code)]
    pub queue: wgpu::Queue,

    pub yuv_bind_group_layout: wgpu::BindGroupLayout,
    pub rgba_bind_group_layout: wgpu::BindGroupLayout,
    pub bgra_bind_group_layout: wgpu::BindGroupLayout,
    pub yuv_to_rgba_converter: YUVToRGBAConverter,
    pub rgba_to_yuv_converter: RGBAToYUVConverter,
    pub bgra_to_rgba_converter: BGRAToRGBAConverter,

    pub shader_parameters_bind_group_layout: wgpu::BindGroupLayout,

    pub compositor_provided_parameters_buffer: wgpu::Buffer,
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
                features: wgpu::Features::TEXTURE_BINDING_ARRAY,
            },
            None,
        ))?;

        let yuv_bind_group_layout = YUVTextures::new_bind_group_layout(&device);
        let rgba_bind_group_layout = RGBATexture::new_bind_group_layout(&device);
        let bgra_bind_group_layout = BGRATexture::new_bind_group_layout(&device);
        let yuv_to_rgba_converter = YUVToRGBAConverter::new(&device, &yuv_bind_group_layout);
        let rgba_to_yuv_converter = RGBAToYUVConverter::new(&device, &rgba_bind_group_layout);
        let bgra_to_rgba_converter = BGRAToRGBAConverter::new(&device, &bgra_bind_group_layout);

        let shader_parameters_bind_group_layout = Shader::new_parameters_bind_group_layout(&device);

        let compositor_provided_parameters_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("global shader parameters buffer"),
            mapped_at_creation: false,
            size: std::mem::size_of::<GlobalShaderParameters>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        // this is temporary until we implement proper error handling in the wgpu renderer
        // it's important to overwrite this though, because the default uncaptured error
        // handler panics
        device.on_uncaptured_error(Box::new(|e| {
            error!("wgpu error: {:?}", e);
        }));

        Ok(Self {
            device,
            queue,
            yuv_bind_group_layout,
            rgba_bind_group_layout,
            bgra_bind_group_layout,
            yuv_to_rgba_converter,
            rgba_to_yuv_converter,
            bgra_to_rgba_converter,
            shader_parameters_bind_group_layout,
            compositor_provided_parameters_buffer,
        })
    }
}

#[repr(C)]
#[derive(Debug, bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct GlobalShaderParameters {
    time: f32,
}

impl GlobalShaderParameters {
    pub fn new(time: Duration) -> Self {
        Self {
            time: time.as_secs_f32(),
        }
    }
}
