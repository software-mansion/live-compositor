use crate::renderer::{texture::NodeTexture, CommonShaderParameters, WgpuError, WgpuErrorScope};

use std::{sync::Arc, time::Duration};

use compositor_common::scene::NodeId;

use crate::renderer::{texture::Texture, WgpuCtx};

use self::pipeline::Pipeline;

pub mod node;
mod pipeline;

const INPUT_TEXTURES_AMOUNT: u32 = 16;

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
    wgpu_ctx: Arc<WgpuCtx>,
    pipeline: Pipeline,
    empty_texture: Texture,
}

impl Shader {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>, shader_src: String) -> Result<Self, WgpuError> {
        let scope = WgpuErrorScope::push(&wgpu_ctx.device);

        let pipeline = Pipeline::new(
            &wgpu_ctx.device,
            wgpu::ShaderSource::Wgsl(shader_src.into()),
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
        target: &NodeTexture,
        pts: Duration,
        clear_color: Option<wgpu::Color>,
    ) {
        let ctx = &self.wgpu_ctx;

        // TODO: sources need to be ordered

        // TODO: most things that happen in this method should not be done every frame
        let textures = sources
            .iter()
            .map(|(_, texture)| texture.rgba_texture())
            .collect::<Vec<_>>();

        let mut texture_views = textures
            .iter()
            .map(|texture| &texture.texture().view)
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
            CommonShaderParameters::new(pts, sources.len() as u32, target.resolution);

        self.pipeline.render(
            &input_textures_bg,
            params,
            target.rgba_texture().texture(),
            ctx,
            common_shader_params,
            clear_color,
        );
    }
}
