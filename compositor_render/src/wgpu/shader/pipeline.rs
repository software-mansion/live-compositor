use std::num::NonZeroU32;

use wgpu::ShaderStages;

use crate::wgpu::{
    common_pipeline::{Sampler, Vertex},
    texture::Texture,
    WgpuCtx,
};

use super::{
    common_params::CommonShaderParameters, INPUT_TEXTURES_AMOUNT, USER_DEFINED_BUFFER_GROUP,
};

#[derive(Debug)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    sampler: Sampler,
    pub(super) textures_bgl: wgpu::BindGroupLayout,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        shader_source: wgpu::ShaderSource,
        uniforms_bgl: &wgpu::BindGroupLayout,
    ) -> Self {
        let sampler = Sampler::new(device);

        let textures_bgl = Self::create_texture_bind_group_layout(device, INPUT_TEXTURES_AMOUNT);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shader transformation pipeline layout"),
            bind_group_layouts: &[&textures_bgl, uniforms_bgl, &sampler.bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX_FRAGMENT,
                range: 0..CommonShaderParameters::push_constant_size(),
            }],
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: shader_source,
        });

        let pipeline = Self::create_render_pipeline(device, &pipeline_layout, &shader_module);

        Self {
            pipeline,
            sampler,
            textures_bgl,
        }
    }

    pub fn create_render_pipeline(
        device: &wgpu::Device,
        pipeline_layout: &wgpu::PipelineLayout,
        shader_module: &wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shader transformation pipeline :^)"),
            depth_stencil: None,
            primitive: wgpu::PrimitiveState {
                conservative: false,
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Fill,
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                unclipped_depth: false,
            },
            vertex: wgpu::VertexState {
                buffers: &[Vertex::LAYOUT],
                module: shader_module,
                entry_point: super::VERTEX_ENTRYPOINT_NAME,
            },
            fragment: Some(wgpu::FragmentState {
                module: shader_module,
                entry_point: super::FRAGMENT_ENTRYPOINT_NAME,
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    write_mask: wgpu::ColorWrites::all(),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                })],
            }),
            layout: Some(pipeline_layout),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
    }

    pub fn create_texture_bind_group_layout(
        device: &wgpu::Device,
        texture_count: u32,
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shader transformation textures bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: NonZeroU32::new(texture_count),
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
            }],
        })
    }

    pub fn render(
        &self,
        inputs: &wgpu::BindGroup,
        uniforms: &wgpu::BindGroup,
        target: &Texture,
        ctx: &WgpuCtx,
        common_parameters: CommonShaderParameters,
        clear_color: Option<wgpu::Color>,
    ) {
        let mut encoder = ctx.device.create_command_encoder(&Default::default());
        let clear_color = clear_color.unwrap_or(wgpu::Color::TRANSPARENT);

        let mut render_plane = |input_id: i32, clear: bool| {
            let load = match clear {
                true => wgpu::LoadOp::Clear(clear_color),
                false => wgpu::LoadOp::Load,
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations { load, store: true },
                    view: &target.view,
                    resolve_target: None,
                })],
                // TODO: depth stencil attachments
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);

            render_pass.set_push_constants(
                ShaderStages::VERTEX_FRAGMENT,
                0,
                common_parameters.push_constant(),
            );

            render_pass.set_bind_group(0, inputs, &[]);
            render_pass.set_bind_group(USER_DEFINED_BUFFER_GROUP, uniforms, &[]);
            render_pass.set_bind_group(2, &self.sampler.bind_group, &[]);

            ctx.plane_cache
                .get_plane(input_id)
                .unwrap()
                .draw(&mut render_pass);
        };

        if common_parameters.texture_count == 0 {
            render_plane(-1, true);
        } else {
            for input_id in 0..common_parameters.texture_count {
                let is_first = input_id == 0;
                render_plane(input_id as i32, is_first);
            }
        }

        // TODO: this should not submit, it should return the command buffer.
        //       the buffer should then be put in an array with other command
        //       buffers and submitted as a whole
        ctx.queue.submit(Some(encoder.finish()));
    }
}
