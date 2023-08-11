use crate::renderer::{
    texture::NodeTexture, GlobalShaderParameters, RegisterTransformationCtx, RenderCtx,
};

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
/// @group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
/// @group(1) @binding(0) var<uniform> shaders_custom_buffer: CustomStruct;
/// @group(1) @binding(1) var<uniform> parameters_received_from_the_compositor: SomeStruct;
/// @group(2) @binding(0) var sampler_: sampler
/// ```
pub struct Shader {
    wgpu_ctx: Arc<WgpuCtx>,
    pipeline: Pipeline,
}

impl Shader {
    pub fn new(ctx: &RegisterTransformationCtx, shader_src: String) -> Self {
        // TODO: Error handling
        let pipeline = Pipeline::new(
            &ctx.wgpu_ctx.device,
            wgpu::ShaderSource::Wgsl(shader_src.into()),
            &ctx.wgpu_ctx.shader_parameters_bind_group_layout,
        );

        Self {
            wgpu_ctx: ctx.wgpu_ctx.clone(),
            pipeline,
        }
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
    ) {
        // TODO: error handling
        let ctx = &self.wgpu_ctx;

        // TODO: sources need to be ordered

        // TODO: most things that happen in this method should not be done every frame
        let empty_texture = Texture::new(
            ctx,
            Some("empty texture"),
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING,
        );

        let textures = sources
            .iter()
            .map(|(_, texture)| texture.rgba_texture())
            .collect::<Vec<_>>();

        let mut texture_views = textures
            .iter()
            .map(|texture| &texture.texture().view)
            .collect::<Vec<_>>();

        texture_views
            .extend((textures.len()..INPUT_TEXTURES_AMOUNT as usize).map(|_| &empty_texture.view));

        // TODO: not sure if this should be done every frame or not
        let input_textures_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.pipeline.textures_bgl,
            label: None,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureViewArray(&texture_views),
            }],
        });

        self.pipeline.render(
            &input_textures_bg,
            params,
            target.rgba_texture().texture(),
            ctx,
        );
    }
}

pub fn prepare_render_loop(ctx: &RenderCtx, pts: Duration, textures_count: u32) {
    let ctx = ctx.wgpu_ctx;

    let new_buffer = GlobalShaderParameters::new(pts, textures_count);

    ctx.queue.write_buffer(
        &ctx.compositor_provided_parameters_buffer,
        0,
        bytemuck::cast_slice(&[new_buffer]),
    );
}
