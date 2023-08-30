use std::sync::Arc;
use std::time::Duration;

use compositor_common::{
    scene::{InputId, OutputId, Resolution, SceneSpec},
    Framerate,
};
use log::error;

use crate::{
    frame_set::FrameSet,
    render_loop::{populate_inputs, read_outputs},
    transformations::{
        image_renderer::ImageError,
        text_renderer::TextRendererCtx,
        web_renderer::chromium::{ChromiumContext, ChromiumContextError},
    },
    WebRendererOptions,
};
use crate::{
    registry::{self},
    render_loop::run_transforms,
    transformations::{shader::Shader, web_renderer::WebRendererNewError},
};

use self::{
    format::TextureFormat,
    renderers::Renderers,
    scene::{Scene, SceneUpdateError},
    utils::TextureUtils,
};

pub mod common_pipeline;
mod format;
pub mod renderers;
pub mod scene;
pub mod texture;
mod utils;

pub(crate) use format::bgra_to_rgba::BGRAToRGBAConverter;

pub struct RendererOptions {
    pub web_renderer: WebRendererOptions,
    pub framerate: Framerate,
    pub stream_fallback_timeout: Duration,
}

pub struct Renderer {
    pub wgpu_ctx: Arc<WgpuCtx>,
    pub text_renderer_ctx: TextRendererCtx,
    pub chromium_context: Arc<ChromiumContext>,

    pub scene: Scene,
    pub scene_spec: Arc<SceneSpec>,

    pub(crate) renderers: Renderers,

    stream_fallback_timeout: Duration,
}

pub struct RenderCtx<'a> {
    pub wgpu_ctx: &'a Arc<WgpuCtx>,

    pub text_renderer_ctx: &'a TextRendererCtx,
    pub chromium: &'a Arc<ChromiumContext>,

    pub(crate) renderers: &'a Renderers,

    pub(crate) stream_fallback_timeout: Duration,
}

pub struct RegisterCtx {
    pub wgpu_ctx: Arc<WgpuCtx>,
    pub chromium: Arc<ChromiumContext>,
}

#[derive(Debug, thiserror::Error)]
pub enum RendererInitError {
    #[error("failed to initialize a wgpu context")]
    FailedToInitWgpuCtx(#[from] WgpuCtxNewError),

    #[error("failed to init chromium context")]
    FailedToInitChromiumCtx(#[from] ChromiumContextError),

    #[error("failed to initialize builtin transformation")]
    BuiltInTransformationsInitError(#[from] WgpuError),
}

#[derive(Debug, thiserror::Error)]
pub enum RendererRegisterError {
    #[error("failed to register a renderer")]
    RendererRegistry(#[from] registry::RegisterError),

    #[error("failed to to initialize the shader")]
    Shader(#[from] WgpuError),

    #[error("failed to create web renderer instance")]
    WebRendererInstance(#[from] WebRendererNewError),

    #[error("failed to prepare image")]
    Image(#[from] ImageError),
}

impl Renderer {
    pub fn new(opts: RendererOptions) -> Result<Self, RendererInitError> {
        let wgpu_ctx = Arc::new(WgpuCtx::new()?);

        Ok(Self {
            wgpu_ctx: wgpu_ctx.clone(),
            text_renderer_ctx: TextRendererCtx::new(),
            chromium_context: Arc::new(ChromiumContext::new(opts.web_renderer, opts.framerate)?),
            scene: Scene::empty(),
            renderers: Renderers::new(wgpu_ctx)?,
            scene_spec: Arc::new(SceneSpec {
                nodes: vec![],
                outputs: vec![],
            }),

            stream_fallback_timeout: opts.stream_fallback_timeout,
        })
    }

    pub(super) fn register_ctx(&self) -> RegisterCtx {
        RegisterCtx {
            wgpu_ctx: self.wgpu_ctx.clone(),
            chromium: self.chromium_context.clone(),
        }
    }

    pub fn render(
        &mut self,
        mut inputs: FrameSet<InputId>,
    ) -> Result<FrameSet<OutputId>, RenderError> {
        let ctx = &mut RenderCtx {
            wgpu_ctx: &self.wgpu_ctx,
            chromium: &self.chromium_context,
            text_renderer_ctx: &self.text_renderer_ctx,
            renderers: &self.renderers,
            stream_fallback_timeout: self.stream_fallback_timeout,
        };

        let scope = WgpuErrorScope::push(&ctx.wgpu_ctx.device);

        populate_inputs(ctx, &mut self.scene, &mut inputs).unwrap();
        run_transforms(ctx, &mut self.scene, inputs.pts).unwrap();
        let frames = read_outputs(ctx, &mut self.scene, inputs.pts).unwrap();

        scope.pop(&ctx.wgpu_ctx.device)?;

        Ok(FrameSet {
            frames,
            pts: inputs.pts,
        })
    }

    pub fn update_scene(&mut self, scene_specs: Arc<SceneSpec>) -> Result<(), SceneUpdateError> {
        self.scene.update(
            &RenderCtx {
                wgpu_ctx: &self.wgpu_ctx,
                text_renderer_ctx: &self.text_renderer_ctx,
                chromium: &self.chromium_context,
                renderers: &self.renderers,
                stream_fallback_timeout: self.stream_fallback_timeout,
            },
            &scene_specs,
        )?;
        self.scene_spec = scene_specs;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("wgpu error encountered while rendering")]
    WgpuError(#[from] WgpuError),
}

pub struct WgpuCtx {
    #[allow(dead_code)]
    pub device: wgpu::Device,

    #[allow(dead_code)]
    pub queue: wgpu::Queue,

    pub format: TextureFormat,
    pub utils: TextureUtils,

    pub shader_parameters_bind_group_layout: wgpu::BindGroupLayout,
    pub compositor_provided_parameters_buffer: wgpu::Buffer,
}

#[derive(Debug, thiserror::Error)]
pub enum WgpuCtxNewError {
    #[error("failed to get a wgpu adapter")]
    NoAdapter,

    #[error("failed to get a wgpu device")]
    NoDevice(#[from] wgpu::RequestDeviceError),

    #[error("wgpu error")]
    WgpuError(#[from] WgpuError),
}

#[derive(Debug, thiserror::Error)]
pub enum WgpuError {
    #[error("wgpu validation error: {0}")]
    Validation(String),
    #[error("wgpu out of memory error: {0}")]
    OutOfMemory(String),
}

impl From<wgpu::Error> for WgpuError {
    fn from(value: wgpu::Error) -> Self {
        match value {
            wgpu::Error::OutOfMemory { .. } => Self::OutOfMemory(format!("{value}")),
            wgpu::Error::Validation { .. } => Self::Validation(format!("{value}")),
        }
    }
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
                limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    ..Default::default()
                },
                features: wgpu::Features::TEXTURE_BINDING_ARRAY
                    | wgpu::Features::PUSH_CONSTANTS
                    | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                    | wgpu::Features::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING,
            },
            None,
        ))?;

        let scope = WgpuErrorScope::push(&device);

        let format = TextureFormat::new(&device);
        let utils = TextureUtils::new(&device);

        let shader_parameters_bind_group_layout = Shader::new_parameters_bind_group_layout(&device);

        let compositor_provided_parameters_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("global shader parameters buffer"),
            mapped_at_creation: false,
            size: std::mem::size_of::<CommonShaderParameters>() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
        });

        scope.pop(&device)?;

        device.on_uncaptured_error(Box::new(|e| {
            error!("wgpu error: {:?}", e);
        }));

        Ok(Self {
            device,
            queue,
            format,
            utils,
            shader_parameters_bind_group_layout,
            compositor_provided_parameters_buffer,
        })
    }
}

#[repr(C)]
#[derive(Debug, bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct CommonShaderParameters {
    time: f32,
    pub textures_count: u32,
    output_resolution: [u32; 2],
}

impl CommonShaderParameters {
    pub fn new(time: Duration, textures_count: u32, output_resolution: Resolution) -> Self {
        Self {
            time: time.as_secs_f32(),
            textures_count,
            output_resolution: [
                output_resolution.width as u32,
                output_resolution.height as u32,
            ],
        }
    }

    pub fn push_constant_size() -> u32 {
        let size = std::mem::size_of::<CommonShaderParameters>() as u32;
        match size % 4 {
            0 => size,
            rest => size + (4 - rest),
        }
    }

    pub fn push_constant(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

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
