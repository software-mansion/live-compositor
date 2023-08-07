use std::{
    cmp::max,
    sync::{Arc, Mutex},
};

use compositor_common::scene::text_params;
use glyphon::{
    AttrsOwned, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache, TextArea, TextAtlas,
    TextBounds,
};
use log::info;
use text_params::TextParams;
use wgpu::{
    CommandEncoderDescriptor, LoadOp, MultisampleState, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, TextureFormat,
};

use crate::renderer::{texture::NodeTexture, RenderCtx};

#[allow(dead_code)]
pub struct TextSpec {
    content: Arc<str>,
    attributes: AttrsOwned,
    font_size: f32,
    line_height: f32,
    align: glyphon::cosmic_text::Align,
    wrap: glyphon::cosmic_text::Wrap,
}

impl From<TextParams> for TextSpec {
    fn from(text_params: TextParams) -> Self {
        Self {
            attributes: Into::into(&text_params),
            content: text_params.content,
            font_size: text_params.font_size,
            line_height: text_params.line_height.unwrap_or(text_params.font_size),
            align: text_params.align.into(),
            wrap: text_params.wrap.into(),
        }
    }
}

#[allow(dead_code)]
pub struct TextRendererCtx {
    font_system: Mutex<FontSystem>,
    swash_cache: Mutex<SwashCache>,
}

impl TextRendererCtx {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            font_system: Mutex::new(FontSystem::new()),
            swash_cache: Mutex::new(SwashCache::new()),
        }
    }
}

impl Default for TextRendererCtx {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub struct TextRenderer {
    text_specs: TextSpec,
    was_rendered: Mutex<bool>,
}

impl TextRenderer {
    #[allow(dead_code)]
    pub fn new(text_params: TextParams) -> Self {
        Self {
            was_rendered: Mutex::new(false),
            text_specs: text_params.into(),
        }
    }

    pub fn render(&self, renderer_ctx: &mut RenderCtx, target: &NodeTexture) {
        let mut was_rendered = self.was_rendered.lock().unwrap();
        if *was_rendered {
            return;
        }

        info!("Text render");
        let font_system = &mut renderer_ctx.text_renderer_ctx.font_system.lock().unwrap();
        let cache = &mut renderer_ctx.text_renderer_ctx.swash_cache.lock().unwrap();

        let swapchain_format = TextureFormat::Rgba8Unorm;
        let mut atlas = TextAtlas::new(
            &renderer_ctx.wgpu_ctx.device,
            &renderer_ctx.wgpu_ctx.queue,
            swapchain_format,
        );
        let mut text_renderer = glyphon::TextRenderer::new(
            &mut atlas,
            &renderer_ctx.wgpu_ctx.device,
            MultisampleState::default(),
            None,
        );
        let mut buffer = Buffer::new(
            font_system,
            Metrics::new(self.text_specs.font_size, self.text_specs.line_height),
        );

        buffer.set_size(
            font_system,
            target.resolution.width as f32,
            target.resolution.height as f32,
        );

        buffer.set_text(
            font_system,
            &self.text_specs.content,
            self.text_specs.attributes.as_attrs(),
            Shaping::Advanced,
        );

        buffer.set_wrap(font_system, self.text_specs.wrap);

        for line in &mut buffer.lines {
            line.set_align(Some(self.text_specs.align));
        }

        buffer.shape_until_scroll(font_system);

        // TODO add different output texture size strategies
        // use this to crop texture to smallest possible size if needed
        // this "cutting to smallest possible size" strategy
        // should require align to left or justify
        let texture_size = Self::get_texture_size(buffer.lines.iter(), self.text_specs.line_height);
        info!("Text rendered size: {:?}", texture_size);

        text_renderer
            .prepare(
                &renderer_ctx.wgpu_ctx.device,
                &renderer_ctx.wgpu_ctx.queue,
                font_system,
                &mut atlas,
                glyphon::Resolution {
                    width: target.resolution.width as u32,
                    height: target.resolution.height as u32,
                },
                [TextArea {
                    buffer: &buffer,
                    left: 0 as f32,
                    top: 0 as f32,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: target.resolution.width as i32,
                        bottom: target.resolution.height as i32,
                    },
                    default_color: Color::rgb(255, 255, 255),
                }],
                cache,
            )
            .unwrap();

        let mut encoder =
            renderer_ctx
                .wgpu_ctx
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Text renderer encoder"),
                });

        let target_texture = &target.rgba_texture();
        let view = &target_texture.texture().view;
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            text_renderer.render(&atlas, &mut pass).unwrap();
        }

        renderer_ctx.wgpu_ctx.queue.submit(Some(encoder.finish()));
        *was_rendered = true;
    }

    fn get_texture_size<'a, I: Iterator<Item = &'a glyphon::BufferLine>>(
        lines: I,
        line_height: f32,
    ) -> (u32, u32) {
        // TODO add different output texture size strategies
        // use this to crop texture to smallest possible size if needed
        let mut width = 0;
        let mut lines_count = 0u32;

        for line in lines {
            if let Some(layout) = line.layout_opt() {
                for layout_line in layout {
                    lines_count += 1;
                    width = max(width, layout_line.w.ceil() as u32);
                }
            }
        }

        let height = lines_count * line_height.ceil() as u32;
        (width, height)
    }
}
