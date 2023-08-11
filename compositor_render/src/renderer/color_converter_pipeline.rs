use wgpu::ShaderStages;

use super::{
    common_pipeline::{RectangleRenderBuffers, Sampler, U32Uniform, Vertex},
    texture::{RGBATexture, Texture, YUVTextures},
    WgpuCtx,
};

const PRIMITIVE_STATE: wgpu::PrimitiveState = wgpu::PrimitiveState {
    polygon_mode: wgpu::PolygonMode::Fill,
    topology: wgpu::PrimitiveTopology::TriangleList,
    front_face: wgpu::FrontFace::Ccw,
    cull_mode: Some(wgpu::Face::Back),
    strip_index_format: None,
    conservative: false,
    unclipped_depth: false,
};

pub struct YUVToRGBAConverter {
    pipeline: wgpu::RenderPipeline,
    sampler: Sampler,
    buffers: RectangleRenderBuffers,
}

impl YUVToRGBAConverter {
    pub fn new(
        device: &wgpu::Device,
        yuv_textures_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("yuv_to_rgba.wgsl"));
        let sampler = Sampler::new(device);
        let buffers = RectangleRenderBuffers::new(device);

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

pub struct RGBAToYUVConverter {
    pipeline: wgpu::RenderPipeline,
    plane_selector: U32Uniform,
    sampler: Sampler,
    buffers: RectangleRenderBuffers,
}

impl RGBAToYUVConverter {
    pub fn new(
        device: &wgpu::Device,
        single_texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let plane_selector = U32Uniform::new(device);
        let sampler = Sampler::new(device);
        let buffers = RectangleRenderBuffers::new(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("RGBA to YUV color converter pipeline layout"),
            bind_group_layouts: &[
                single_texture_bind_group_layout,
                &sampler.bind_group_layout,
                &plane_selector.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("rgba_to_yuv.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("RGBA to YUV color converter pipeline"),
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

        Self {
            pipeline,
            sampler,
            buffers,
            plane_selector,
        }
    }

    pub fn convert(&self, ctx: &WgpuCtx, src: (&RGBATexture, &wgpu::BindGroup), dst: &YUVTextures) {
        for plane in [0, 1, 2] {
            ctx.queue.write_buffer(
                &self.plane_selector.buffer,
                0,
                bytemuck::cast_slice(&[plane as u32]),
            );

            let mut encoder = ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("RGBA to YUV color converter command encoder"),
                });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("YUV to RGBA color converter render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        ops: wgpu::Operations {
                            // We want the background to be black. Black in YUV is y = 0, u = 0.5, v = 0.5
                            // Therefore, we set the clear color to 0, 0, 0 when drawing the y plane
                            // and to 0.5, 0.5, 0.5 when drawing the u and v planes.
                            load: wgpu::LoadOp::Clear(if plane == 0 {
                                wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }
                            } else {
                                wgpu::Color {
                                    r: 0.5,
                                    g: 0.5,
                                    b: 0.5,
                                    a: 1.0,
                                }
                            }),
                            store: true,
                        },
                        view: &dst.plane(plane).view,
                        resolve_target: None,
                    })],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, src.1, &[]);
                render_pass.set_bind_group(1, &self.sampler.bind_group, &[]);
                render_pass.set_bind_group(2, &self.plane_selector.bind_group, &[]);
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
}

pub struct R8FillWithValuePipeline {
    pipeline: wgpu::RenderPipeline,
    buffers: RectangleRenderBuffers,
}

impl R8FillWithValuePipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("r8_fill_color.wgsl"));
        let buffers = RectangleRenderBuffers::new(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fill with color render pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::FRAGMENT,
                range: 0..4,
            }],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fill with color pipeline"),
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
                label: Some("Fill R8 texture with color command encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Fill R8 texture with color render pass"),
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
            render_pass.set_push_constants(
                ShaderStages::FRAGMENT,
                0,
                bytemuck::cast_slice(&[value]),
            );
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
