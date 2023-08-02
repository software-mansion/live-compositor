use crate::renderer::{texture::NodeTexture, RenderCtx};

use std::{collections::HashMap, sync::Arc};

use compositor_common::scene::{NodeId, ShaderParams};
use wgpu::util::DeviceExt;

use crate::renderer::{texture::Texture, WgpuCtx};

use self::pipeline::Pipeline;

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
    pub fn new(ctx: &RenderCtx, shader_src: String) -> Self {
        // TODO: Error handling
        let pipeline = Pipeline::new(
            &ctx.wgpu_ctx.device,
            wgpu::ShaderSource::Wgsl(shader_src.into()),
        );

        Self {
            wgpu_ctx: ctx.wgpu_ctx.clone(),
            pipeline,
        }
    }

    pub fn render(
        &self,
        _params: &HashMap<String, ShaderParams>,
        sources: &[(&NodeId, &NodeTexture)],
        target: &NodeTexture,
    ) {
        // TODO: error handling
        let ctx = &self.wgpu_ctx;

        // TODO: sources need to be ordered

        // TODO: most things that happen in this method should not be done every frame
        let buffer_user = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                // TODO: fill the contents
                contents: &[0, 0, 0, 0],
                label: Some("user's custom buffer"),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let buffer_compositor = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                // TODO: fill the contents
                contents: &[0, 0, 0, 0],
                label: Some("compositor-provided buffer"),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let uniform_bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.pipeline.bgls.uniforms_bgl,
            label: None,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer_user,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer_compositor,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

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
            layout: &self.pipeline.bgls.textures_bgl,
            label: None,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureViewArray(&texture_views),
            }],
        });

        self.pipeline.render(
            &input_textures_bg,
            &uniform_bg,
            target.rgba_texture().texture(),
            ctx,
        );
    }
}
