use crate::renderer::{
    texture::NodeTextureState, CommonShaderParameters, WgpuError, WgpuErrorScope,
};

use std::{sync::Arc, time::Duration};
use wgpu::util::DeviceExt;

use compositor_common::scene::shader::ShaderParam;

use crate::renderer::{texture::Texture, WgpuCtx};

use self::{
    error::{ParametersValidationError, ShaderValidationError},
    pipeline::Pipeline,
    validation::{validate_contains_header, validate_params},
};

pub mod error;
mod pipeline;
pub mod validation;

const INPUT_TEXTURES_AMOUNT: u32 = 16;

pub const VERTEX_ENTRYPOINT_NAME: &str = "vs_main";
pub const FRAGMENT_ENTRYPOINT_NAME: &str = "fs_main";

pub const USER_DEFINED_BUFFER_GROUP: u32 = 1;
pub const USER_DEFINED_BUFFER_BINDING: u32 = 0;

#[derive(Debug, thiserror::Error)]
pub enum CreateShaderError {
    #[error(transparent)]
    Wgpu(#[from] WgpuError),

    #[error(transparent)]
    Validation(#[from] ShaderValidationError),

    #[error("Shader parse error: {0}")]
    ParseError(naga::front::wgsl::ParseError),
}

/// Abstraction over single GPU shader. Used for builtins and shaders.
///
/// The bind group layout for the shader:
///
/// ```wgsl
/// var<push_constant> common_params: CommonShaderParameters;
///
/// @group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
/// @group(1) @binding(0) var<uniform> shaders_custom_buffer: CustomStruct;
/// @group(2) @binding(0) var sampler_: sampler;
/// ```
#[derive(Debug)]
pub struct GpuShader {
    pub wgpu_ctx: Arc<WgpuCtx>,
    pipeline: Pipeline,
    empty_texture: Texture,
    shader: naga::Module,
}

impl GpuShader {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>, shader_src: String) -> Result<Self, CreateShaderError> {
        let scope = WgpuErrorScope::push(&wgpu_ctx.device);

        let shader =
            naga::front::wgsl::parse_str(&shader_src).map_err(CreateShaderError::ParseError)?;

        validate_contains_header(&wgpu_ctx.shader_header, &shader)?;

        let pipeline = Pipeline::new(
            &wgpu_ctx.device,
            wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(shader.clone())),
            &wgpu_ctx.shader_parameters_bind_group_layout,
        );

        let empty_texture = Texture::new(
            wgpu_ctx,
            Some("empty texture"),
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING,
        );

        scope.pop(&wgpu_ctx.device)?;

        Ok(Self {
            wgpu_ctx: wgpu_ctx.clone(),
            pipeline,
            empty_texture,
            shader,
        })
    }

    pub fn new_parameters_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shader parameters bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            }],
        })
    }

    pub fn render(
        &self,
        params: &wgpu::BindGroup,
        textures: &[Option<&Texture>],
        target: &NodeTextureState,
        pts: Duration,
        clear_color: Option<wgpu::Color>,
    ) {
        let ctx = &self.wgpu_ctx;

        // TODO: sources need to be ordered

        // TODO: most things that happen in this method should not be done every frame

        let mut texture_views = textures
            .iter()
            .map(|texture| match texture {
                Some(texture) => &texture.view,
                None => &self.empty_texture.view,
            })
            .collect::<Vec<_>>();

        texture_views.extend(
            (textures.len()..INPUT_TEXTURES_AMOUNT as usize).map(|_| &self.empty_texture.view),
        );

        let input_textures_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.pipeline.textures_bgl,
            label: None,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureViewArray(&texture_views),
            }],
        });

        let common_shader_params =
            CommonShaderParameters::new(pts, textures.len() as u32, target.resolution());

        self.pipeline.render(
            &input_textures_bg,
            params,
            target.rgba_texture().texture(),
            ctx,
            common_shader_params,
            clear_color,
        );
    }

    pub fn validate_params(&self, params: &ShaderParam) -> Result<(), ParametersValidationError> {
        let ty = self
            .shader
            .global_variables
            .iter()
            .find(|(_, global)| match global.binding.as_ref() {
                Some(binding) => {
                    (binding.group, binding.binding)
                        == (USER_DEFINED_BUFFER_GROUP, USER_DEFINED_BUFFER_BINDING)
                }

                None => false,
            })
            .map(|(_, handle)| handle.ty)
            .ok_or(ParametersValidationError::NoBindingInShader)?;

        validate_params(params, ty, &self.shader)
    }
}

pub(super) struct ParamsBuffer {
    pub bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
    content: bytes::Bytes,
}

impl ParamsBuffer {
    pub fn new(content: bytes::Bytes, wgpu_ctx: &WgpuCtx) -> Self {
        let content_or_zero = match content.is_empty() {
            true => bytes::Bytes::copy_from_slice(&[0]),
            false => content.clone(),
        };

        let buffer = wgpu_ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("params buffer"),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                contents: &content_or_zero,
            });

        let bind_group = wgpu_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("params bind group"),
                layout: &wgpu_ctx.shader_parameters_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        Self {
            bind_group,
            buffer,
            content,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn update(&mut self, content: bytes::Bytes, wgpu_ctx: &WgpuCtx) {
        if self.content.len() != content.len() {
            *self = Self::new(content, wgpu_ctx);
            return;
        }

        if self.content != content {
            wgpu_ctx.queue.write_buffer(&self.buffer, 0, &content);
        }
    }
}
