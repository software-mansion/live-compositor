use std::sync::Arc;

use wgpu::ShaderSource;

use crate::{
    renderer::render_graph::NodeId,
    wgpu::{
        common_pipeline::Sampler,
        shader::{pipeline, CreateShaderError},
        texture::{NodeTexture, NodeTextureState, Texture},
        WgpuCtx, WgpuErrorScope,
    },
};

pub(super) const INPUT_TEXTURES_AMOUNT: u32 = 16;

#[derive(Debug)]
pub struct LayoutShader {
    pipeline: wgpu::RenderPipeline,
    sampler: Sampler,
    textures_bgl: wgpu::BindGroupLayout,

    empty_texture: Texture,
}

impl LayoutShader {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, CreateShaderError> {
        let shader_src = include_str!("./apply_layouts.wgsl");

        let scope = WgpuErrorScope::push(&wgpu_ctx.device);

        let shader_module =
            naga::front::wgsl::parse_str(shader_src).map_err(CreateShaderError::ParseError)?;
        let result = Self::new_pipeline(
            wgpu_ctx,
            wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(shader_module.clone())),
        )?;

        scope.pop(&wgpu_ctx.device)?;

        Ok(result)
    }

    fn new_pipeline(
        wgpu_ctx: &Arc<WgpuCtx>,
        shader_src: ShaderSource,
    ) -> Result<Self, CreateShaderError> {
        let sampler = Sampler::new(&wgpu_ctx.device);

        let textures_bgl = pipeline::Pipeline::create_texture_bind_group_layout(
            &wgpu_ctx.device,
            INPUT_TEXTURES_AMOUNT,
        );

        let pipeline_layout =
            wgpu_ctx
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("shader transformation pipeline layout"),
                    bind_group_layouts: &[
                        &textures_bgl,
                        &wgpu_ctx.shader_parameters_bind_group_layout,
                        &sampler.bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let shader_module = wgpu_ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: shader_src,
            });

        let pipeline = pipeline::Pipeline::create_render_pipeline(
            &wgpu_ctx.device,
            &pipeline_layout,
            &shader_module,
        );

        let empty_texture = Texture::empty(wgpu_ctx);

        Ok(Self {
            pipeline,
            sampler,
            textures_bgl,
            empty_texture,
        })
    }

    pub fn render(
        &self,
        wgpu_ctx: &Arc<WgpuCtx>,
        params: &wgpu::BindGroup,
        sources: &[(&NodeId, &NodeTexture)],
        target: &NodeTextureState,
        layout_count: u32,
    ) {
        let mut texture_views = sources
            .iter()
            .map(|(_id, texture)| match texture.state() {
                Some(texture) => &texture.rgba_texture().texture().view,
                None => &self.empty_texture.view,
            })
            .collect::<Vec<_>>();

        texture_views.extend(
            (texture_views.len()..INPUT_TEXTURES_AMOUNT as usize).map(|_| &self.empty_texture.view),
        );

        let input_textures_bg = wgpu_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.textures_bgl,
                label: None,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&texture_views),
                }],
            });

        let mut encoder = wgpu_ctx.device.create_command_encoder(&Default::default());

        for layout_id in 0..layout_count {
            let load = match layout_id {
                0 => wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                _ => wgpu::LoadOp::Load,
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations { load, store: true },
                    view: &target.rgba_texture().texture().view,
                    resolve_target: None,
                })],
                // TODO: depth stencil attachments
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);

            render_pass.set_bind_group(0, &input_textures_bg, &[]);
            render_pass.set_bind_group(1, params, &[]);
            render_pass.set_bind_group(2, &self.sampler.bind_group, &[]);

            wgpu_ctx
                .plane_cache
                .plane(layout_id as i32)
                .unwrap()
                .draw(&mut render_pass);
        }
        wgpu_ctx.queue.submit(Some(encoder.finish()));
    }
}
