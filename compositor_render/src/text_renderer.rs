use compositor_common::scene::Resolution;
use glyphon::{
    Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, SwashCache, TextArea, TextAtlas,
    TextBounds, TextRenderer,
};
use wgpu::{
    CommandEncoderDescriptor, LoadOp, MultisampleState, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, TextureFormat,
};

use crate::renderer::{texture::RGBATexture, WgpuCtx};

pub fn render_on_frame(wgpu_ctx: &WgpuCtx, resolution: Resolution, dst: &RGBATexture) {
    let swapchain_format = TextureFormat::Rgba8Unorm;
    let mut font_system = FontSystem::new();
    let mut cache = SwashCache::new();
    let mut atlas = TextAtlas::new(&wgpu_ctx.device, &wgpu_ctx.queue, swapchain_format);
    let mut text_renderer = TextRenderer::new(
        &mut atlas,
        &wgpu_ctx.device,
        MultisampleState::default(),
        None,
    );
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

    buffer.set_size(
        &mut font_system,
        resolution.width as f32,
        resolution.height as f32,
    );
    buffer.set_text(
        &mut font_system,
        " Hello world! 👋\n 😃🥹😄😋🦁\n Video Compositor 🚀",
        Attrs::new().family(Family::SansSerif),
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
                width: resolution.width as u32,
                height: resolution.height as u32,
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
    // let text_frame = RGBATexture::new(&wgpu_ctx, &resolution);
    // let view = text_frame.0.view;

    {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &dst.0.view,
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
}

