use compositor_common::scene::Resolution;
use glyphon::{
    AttrsOwned, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache, TextArea, TextAtlas,
    TextBounds,
};
use wgpu::{
    CommandEncoderDescriptor, LoadOp, MultisampleState, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, TextureFormat,
};

use crate::renderer::{
    common_pipeline::{Sampler, Vertex},
    texture::RGBATexture,
    WgpuCtx,
};

#[allow(dead_code)]
pub struct Box {
    top_left_corner: (u32, u32),
    width: u32,
    height: u32,
}

#[allow(dead_code)]
pub struct TextSpec {
    content: String,
    text_box: Box,
    attributes: AttrsOwned,
}

#[allow(dead_code)]
pub struct TextRendererCtx {
    blending_pipeline: wgpu::RenderPipeline,
}

impl TextRendererCtx {
    #[allow(dead_code)]
    pub fn new(wgpu_ctx: &WgpuCtx) -> Self {
        let sampler = Sampler::new(&wgpu_ctx.device);

        let shader_module = wgpu_ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("./text_renderer/text_blending.wgsl"));

        let blending_pipeline_layout =
            wgpu_ctx
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Text renderer blending pipeline layout"),
                    bind_group_layouts: &[
                        &wgpu_ctx.rgba_bind_group_layout,
                        &sampler.bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let blending_pipeline: wgpu::RenderPipeline =
            wgpu_ctx
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Text renderer blending pipeline"),
                    layout: Some(&blending_pipeline_layout),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        strip_index_format: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: "vs_main",
                        buffers: &[Vertex::LAYOUT],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            blend: Some(wgpu::BlendState::REPLACE), // REPLACE to keep transparent parts not blended with cleared background
                            write_mask: wgpu::ColorWrites::all(),
                            format: wgpu::TextureFormat::Rgba8Unorm,
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

        Self { blending_pipeline }
    }
}

#[allow(dead_code)]
pub struct TextRenderer {
    text_frame: RGBATexture,
}

impl TextRenderer {
    #[allow(dead_code)]
    pub fn new(wgpu_ctx: &WgpuCtx, output_resolution: Resolution, text: TextSpec) -> Self {
        Self {
            text_frame: Self::render_text(wgpu_ctx, output_resolution, text),
        }
    }

    fn render_text(
        wgpu_ctx: &WgpuCtx,
        output_resolution: Resolution,
        text: TextSpec,
    ) -> RGBATexture {
        let swapchain_format = TextureFormat::Rgba8Unorm;
        let mut font_system = FontSystem::new();
        let mut cache = SwashCache::new();
        let mut atlas = TextAtlas::new(&wgpu_ctx.device, &wgpu_ctx.queue, swapchain_format);
        let mut text_renderer = glyphon::TextRenderer::new(
            &mut atlas,
            &wgpu_ctx.device,
            MultisampleState::default(),
            None,
        );
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

        buffer.set_size(
            &mut font_system,
            output_resolution.width as f32,
            output_resolution.height as f32,
        );

        buffer.set_text(
            &mut font_system,
            &text.content,
            text.attributes.as_attrs(),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(&mut font_system);

        text_renderer
            .prepare(
                &wgpu_ctx.device,
                &wgpu_ctx.queue,
                &mut font_system,
                &mut atlas,
                glyphon::Resolution {
                    width: output_resolution.width as u32,
                    height: output_resolution.height as u32,
                },
                [TextArea {
                    buffer: &buffer,
                    left: 100.0,
                    top: 100.0,
                    scale: 3.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 1200,
                        bottom: 600,
                    },
                    default_color: Color::rgb(255, 255, 255),
                }],
                &mut cache,
            )
            .unwrap();

        let mut encoder = wgpu_ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Text renderer encoder"),
            });
        let text_frame = RGBATexture::new(wgpu_ctx, &output_resolution);

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &text_frame.0.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            text_renderer.render(&atlas, &mut pass).unwrap();
        }

        wgpu_ctx.queue.submit(Some(encoder.finish()));
        text_frame
    }

    // pub fn apply(&self, text_renderer_ctx: TextRendererCtx, wgpu_ctx: &WgpuCtx, src: &RGBATexture, dst: &RGBATexture) {
    //     let mut encoder = wgpu_ctx
    //         .device
    //         .create_command_encoder(&CommandEncoderDescriptor {
    //             label: Some("Text renderer blending encoder"),
    //         });
    //     {
    //         let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    //             label: Some("Text renderer blending render pass"),
    //             color_attachments: &[Some(wgpu::RenderPassColorAttachment {
    //                 ops: wgpu::Operations {
    //                     load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
    //                     store: true,
    //                 },
    //                 view: &dst.0.view,
    //                 resolve_target: None,
    //             })],
    //             depth_stencil_attachment: None,
    //         });

    //         render_pass.set_pipeline(&text_renderer_ctx.blending_pipeline);
    //         render_pass.set_bind_group(0, , &[]);
    //         render_pass.set_bind_group(1, &self.common.sampler_bind_group, &[]);
    //         render_pass.set_bind_group(2, &self.uniform_bind_group, &[]);
    //         render_pass.set_vertex_buffer(0, self.common.vertex_buffer.slice(..));
    //         render_pass.set_index_buffer(
    //             self.common.index_buffer.slice(..),
    //             wgpu::IndexFormat::Uint16,
    //         );
    //         render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    //     }

    //     queue.submit(Some(encoder.finish()));
    // }
}
