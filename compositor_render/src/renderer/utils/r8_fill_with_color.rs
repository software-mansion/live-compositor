use wgpu::ShaderStages;

use crate::renderer::{
    common_pipeline::{RectangleRenderBuffers, Vertex, PRIMITIVE_STATE},
    texture::Texture,
    WgpuCtx,
};

pub struct R8FillWithValue {
    pipeline: wgpu::RenderPipeline,
    buffers: RectangleRenderBuffers,
}

impl R8FillWithValue {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("r8_fill_value.wgsl"));
        let buffers = RectangleRenderBuffers::new(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fill with value render pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::FRAGMENT,
                range: 0..4,
            }],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fill with value pipeline"),
            layout: Some(&pipeline_layout),
            primitive: PRIMITIVE_STATE,
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[Vertex::LAYOUT],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::R8Unorm,
                    write_mask: wgpu::ColorWrites::all(),
                    blend: None,
                })],
            }),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self { pipeline, buffers }
    }

    pub fn fill(&self, ctx: &WgpuCtx, dst: &Texture, value: f32) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Fill R8 texture with value command encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Fill R8 texture with value render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                    view: &dst.view,
                    resolve_target: None,
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_push_constants(ShaderStages::FRAGMENT, 0, bytemuck::bytes_of(&value));
            render_pass.set_vertex_buffer(0, self.buffers.vertex.slice(..));
            render_pass.set_index_buffer(
                self.buffers.index.slice(..),
                RectangleRenderBuffers::INDEX_FORMAT,
            );
            render_pass.draw_indexed(0..RectangleRenderBuffers::INDICES.len() as u32, 0, 0..1);
        }

        ctx.queue.submit(Some(encoder.finish()));
    }
}
