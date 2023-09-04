use crate::renderer::{
    texture::{NodeTexture, NodeTextureState},
    CommonShaderParameters, WgpuError, WgpuErrorScope,
};

use std::{sync::Arc, time::Duration};

use compositor_common::{renderer_spec::FallbackStrategy, scene::NodeId};

use crate::renderer::{texture::Texture, WgpuCtx};

use self::{
    pipeline::Pipeline,
    validation::{validate_contains_header, ShaderValidationError},
};

pub mod node;
mod pipeline;
pub mod validation;

const INPUT_TEXTURES_AMOUNT: u32 = 16;

pub const VERTEX_ENTRYPOINT_NAME: &str = "vs_main";
pub const FRAGMENT_ENTRYPOINT_NAME: &str = "fs_main";

#[derive(Debug, thiserror::Error)]
pub enum CreateShaderError {
    #[error(transparent)]
    Wgpu(#[from] WgpuError),

    #[error(transparent)]
    Validation(#[from] ShaderValidationError),

    #[error("Shader parse error:\n{0}")]
    ParseError(#[from] naga::front::wgsl::ParseError),
}

/// The bind group layout for the shader:
///
/// ```wgsl
/// var<push_constant> common_params: CommonShaderParameters;
///
/// @group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
/// @group(1) @binding(0) var<uniform> shaders_custom_buffer: CustomStruct;
/// @group(2) @binding(0) var sampler_: sampler;
/// ```
pub struct Shader {
    pub wgpu_ctx: Arc<WgpuCtx>,
    pipeline: Pipeline,
    empty_texture: Texture,
    fallback_strategy: FallbackStrategy,
}

impl Shader {
    pub fn new(
        wgpu_ctx: &Arc<WgpuCtx>,
        shader_src: String,
        fallback_strategy: FallbackStrategy,
    ) -> Result<Self, CreateShaderError> {
        let scope = WgpuErrorScope::push(&wgpu_ctx.device);

        let shader = naga::front::wgsl::parse_str(&shader_src)?;

        validate_contains_header(&wgpu_ctx.shader_header, &shader)?;

        let pipeline = Pipeline::new(
            &wgpu_ctx.device,
            wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(shader)),
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
            fallback_strategy,
        })
    }

    pub fn new_parameters_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shader parameters bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
            ],
        })
    }

    pub fn render(
        &self,
        params: &wgpu::BindGroup,
        sources: &[(&NodeId, &NodeTexture)],
        target: &NodeTextureState,
        pts: Duration,
        clear_color: Option<wgpu::Color>,
    ) {
        let ctx = &self.wgpu_ctx;

        // TODO: sources need to be ordered

        // TODO: most things that happen in this method should not be done every frame

        let textures = sources
            .iter()
            .map(|(_, node_texture)| node_texture.state())
            .collect::<Vec<_>>();
        let mut texture_views = textures
            .iter()
            .map(|node_texture| match node_texture {
                Some(node_texture) => &node_texture.rgba_texture().texture().view,
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
            CommonShaderParameters::new(pts, sources.len() as u32, target.resolution());

        self.pipeline.render(
            &input_textures_bg,
            params,
            target.rgba_texture().texture(),
            ctx,
            common_shader_params,
            clear_color,
        );
    }

    pub fn fallback_strategy(&self) -> FallbackStrategy {
        self.fallback_strategy
    }
}
