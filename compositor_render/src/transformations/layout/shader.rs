use std::sync::Arc;

use wgpu::{ShaderSource, ShaderStages};

use crate::wgpu::{
    common_pipeline::Sampler,
    shader::{pipeline, CreateShaderError},
    texture::{NodeTexture, NodeTextureState, Texture},
    WgpuCtx, WgpuErrorScope,
};

#[derive(Debug)]
pub struct LayoutShader {
    pipeline: wgpu::RenderPipeline,
    sampler: Sampler,
    texture_bgl: wgpu::BindGroupLayout,

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

        let texture_bgl = wgpu_ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Layout texture bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
            }],
        });

        let pipeline_layout =
            wgpu_ctx
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("shader transformation pipeline layout"),
                    bind_group_layouts: &[
                        &texture_bgl,
                        &wgpu_ctx.shader_parameters_bind_group_layout,
                        &sampler.bind_group_layout,
                    ],
                    push_constant_ranges: &[wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        range: 0..4,
                    }],
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
            texture_bgl,
            empty_texture,
        })
    }

    pub fn render(
        &self,
        wgpu_ctx: &Arc<WgpuCtx>,
        params: &wgpu::BindGroup,
        textures: &[Option<&NodeTexture>],
        target: &NodeTextureState,
    ) {
        let input_texture_bgs: Vec<wgpu::BindGroup> = textures
            .iter()
            .map(|texture| {
                texture
                    .and_then(|texture| texture.state())
                    .map(|state| &state.rgba_texture().texture().view)
                    .unwrap_or(&self.empty_texture.view)
            })
            .map(|view| {
                wgpu_ctx
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        layout: &self.texture_bgl,
                        label: None,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(view),
                        }],
                    })
            })
            .collect();

        let mut encoder = wgpu_ctx.device.create_command_encoder(&Default::default());
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                    view: &target.rgba_texture().texture().view,
                    resolve_target: None,
                })],
                // TODO: depth stencil attachments
                depth_stencil_attachment: None,
            });

            for (layout_id, texture_bg) in input_texture_bgs.iter().enumerate() {
                render_pass.set_pipeline(&self.pipeline);

                render_pass.set_push_constants(
                    ShaderStages::VERTEX_FRAGMENT,
                    0,
                    &(layout_id as u32).to_le_bytes(),
                );

                render_pass.set_bind_group(0, texture_bg, &[]);
                render_pass.set_bind_group(1, params, &[]);
                render_pass.set_bind_group(2, &self.sampler.bind_group, &[]);

                wgpu_ctx
                    .plane_cache
                    .plane(layout_id as i32)
                    .unwrap()
                    .draw(&mut render_pass);
            }
        }
        wgpu_ctx.queue.submit(Some(encoder.finish()));
    }
}
