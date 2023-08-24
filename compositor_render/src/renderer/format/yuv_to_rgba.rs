use crate::renderer::{
    common_pipeline::{InputTexturesPlanes, Sampler, Vertex, PRIMITIVE_STATE},
    texture::{RGBATexture, YUVTextures},
};

use super::WgpuCtx;

pub struct YUVToRGBAConverter {
    pipeline: wgpu::RenderPipeline,
    sampler: Sampler,
    buffers: InputTexturesPlanes,
}

impl YUVToRGBAConverter {
    pub fn new(
        device: &wgpu::Device,
        yuv_textures_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("yuv_to_rgba.wgsl"));
        let sampler = Sampler::new(device);
        let buffers = InputTexturesPlanes::new(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("YUV to RGBA color converter render pipeline layout"),
            bind_group_layouts: &[yuv_textures_bind_group_layout, &sampler.bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("YUV to RGBA color converter render pipeline"),
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
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    write_mask: wgpu::ColorWrites::all(),
                    blend: None,
                })],
            }),

            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            depth_stencil: None,
        });

        Self {
            pipeline,
            sampler,
            buffers,
        }
    }

    pub fn convert(&self, ctx: &WgpuCtx, src: (&YUVTextures, &wgpu::BindGroup), dst: &RGBATexture) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("YUV to RGBA color converter encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("YUV to RGBA color converter render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                    view: &dst.texture().view,
                    resolve_target: None,
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, src.1, &[]);
            render_pass.set_bind_group(1, &self.sampler.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.buffers.vertices(1));
            render_pass
                .set_index_buffer(self.buffers.indices(1), InputTexturesPlanes::INDEX_FORMAT);
            render_pass.draw_indexed(0..InputTexturesPlanes::indices_len(1), 0, 0..1);
        }

        ctx.queue.submit(Some(encoder.finish()));
    }
}
