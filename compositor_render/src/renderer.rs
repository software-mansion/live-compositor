use std::sync::Arc;
use std::time::Duration;

use compositor_common::{
    scene::{constraints::Constraints, InputId, NodeParams, OutputId, Resolution, SceneSpec},
    Framerate,
};
use log::error;

use crate::render_loop::run_transforms;
use crate::{
    error::{InitRendererEngineError, RenderSceneError},
    frame_set::FrameSet,
    render_loop::{populate_inputs, read_outputs},
    transformations::{
        shader_executor::ShaderExecutor, text_renderer::TextRendererCtx,
        web_renderer::chromium_context::ChromiumContext,
    },
    WebRendererOptions,
};

use self::{
    format::TextureFormat,
    renderers::Renderers,
    scene::{Scene, UpdateSceneError},
    utils::TextureUtils,
};

pub mod common_pipeline;
mod format;
pub mod node;
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

impl Renderer {
    pub fn new(opts: RendererOptions) -> Result<Self, InitRendererEngineError> {
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
    ) -> Result<FrameSet<OutputId>, RenderSceneError> {
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

    pub fn update_scene(&mut self, scene_spec: Arc<SceneSpec>) -> Result<(), UpdateSceneError> {
        self.validate_constraints(&scene_spec)?;
        self.scene.update(
            &RenderCtx {
                wgpu_ctx: &self.wgpu_ctx,
                text_renderer_ctx: &self.text_renderer_ctx,
                chromium: &self.chromium_context,
                renderers: &self.renderers,
                stream_fallback_timeout: self.stream_fallback_timeout,
            },
            &scene_spec,
        )?;
        self.scene_spec = scene_spec;
        Ok(())
    }

    fn validate_constraints(&self, scene_spec: &SceneSpec) -> Result<(), UpdateSceneError> {
        let node_constraints = scene_spec
            .nodes
            .iter()
            .map(|node| (node, self.node_constraints(&node.params)));

        for (node_spec, node_constraints) in node_constraints {
            if let Some(node_constraints) = node_constraints {
                node_constraints
                    .validate(scene_spec, &node_spec.node_id)
                    .map_err(|err| {
                        UpdateSceneError::ConstraintsValidationError(err, node_spec.node_id.clone())
                    })?;
            }
        }

        Ok(())
    }

    fn node_constraints(&self, node_params: &NodeParams) -> Option<&Constraints> {
        match node_params {
            NodeParams::WebRenderer { instance_id } => self
                .renderers
                .web_renderers
                .get_ref(instance_id)
                .map(|web_renderer| web_renderer.constrains()),
            NodeParams::Shader { shader_id, .. } => self
                .renderers
                .shaders
                .get_ref(shader_id)
                .map(|shader| shader.constraints()),
            NodeParams::Text(_) => Some(&NodeParams::TEXT_CONSTRAINTS),
            NodeParams::Image { .. } => Some(&NodeParams::IMAGE_CONSTRAINTS),
            NodeParams::Builtin { transformation } => Some(transformation.constrains()),
        }
    }
}

#[derive(Debug)]
pub struct WgpuCtx {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    pub shader_header: naga::Module,

    pub format: TextureFormat,
    pub utils: TextureUtils,

    pub shader_parameters_bind_group_layout: wgpu::BindGroupLayout,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateWgpuCtxError {
    #[error("Failed to get a wgpu adapter.")]
    NoAdapter,

    #[error("Failed to get a wgpu device.")]
    NoDevice(#[from] wgpu::RequestDeviceError),

    #[error(transparent)]
    WgpuError(#[from] WgpuError),
}

#[derive(Debug, thiserror::Error)]
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

impl WgpuCtx {
    fn new() -> Result<Self, CreateWgpuCtxError> {
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
            .ok_or(CreateWgpuCtxError::NoAdapter)?;

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

        let shader_header =
            naga::front::wgsl::parse_str(include_str!("transformations/shader_header.wgsl"))
                .expect("failed to parse the shader header file");

        let scope = WgpuErrorScope::push(&device);

        let format = TextureFormat::new(&device);
        let utils = TextureUtils::new(&device);

        let shader_parameters_bind_group_layout =
            ShaderExecutor::new_parameters_bind_group_layout(&device);

        scope.pop(&device)?;

        device.on_uncaptured_error(Box::new(|e| {
            error!("wgpu error: {:?}", e);
        }));

        Ok(Self {
            device,
            queue,
            shader_header,
            format,
            utils,
            shader_parameters_bind_group_layout,
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
