use compositor_common::scene::{Box, TextParams};
use glyphon::{
    AttrsOwned, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache, TextArea, TextAtlas,
    TextBounds,
};
use wgpu::{
    CommandEncoderDescriptor, LoadOp, MultisampleState, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, TextureFormat,
};

use crate::renderer::{texture::NodeTexture, RenderCtx};

#[allow(dead_code)]
pub struct TextSpec {
    content: String,
    text_box: Box,
    attributes: AttrsOwned,
    font_size: f32,
    line_height: f32,
}

impl From<TextParams> for TextSpec {
    fn from(text_params: TextParams) -> Self {
        let attributes = Self::get_attrs_owned(&text_params);
        let font_size = text_params.font_size;
        let line_height = text_params.line_height.unwrap_or(font_size);

        Self {
            content: text_params.content,
            text_box: text_params.placement,
            attributes,
            font_size,
            line_height,
        }
    }
}

impl TextSpec {
    fn get_attrs_owned(text_params: &TextParams) -> AttrsOwned {
        let color = match text_params.color_rgba {
            Some((r, g, b, a)) => glyphon::Color::rgba(r, g, b, a),
            None => glyphon::Color::rgb(255, 255, 255),
        };

        let family = match &text_params.font_family {
            Some(font_family_name) => glyphon::FamilyOwned::Name(font_family_name.clone()),
            None => glyphon::FamilyOwned::SansSerif,
        };

        let style = match text_params.style {
            Some(compositor_common::scene::Style::Normal) | None => glyphon::Style::Normal,
            Some(compositor_common::scene::Style::Italic) => glyphon::Style::Italic,
            Some(compositor_common::scene::Style::Oblique) => glyphon::Style::Oblique,
        };

        AttrsOwned {
            color_opt: Some(color),
            family_owned: family,
            stretch: Default::default(),
            style,
            weight: Default::default(),
            metadata: Default::default(),
        }
    }
}

#[allow(dead_code)]
pub struct TextRendererCtx {
    font_system: FontSystem,
}

impl TextRendererCtx {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            font_system: FontSystem::new(),
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
    was_rendered: bool,
    text_specs: TextSpec,
}

impl TextRenderer {
    #[allow(dead_code)]
    pub fn new(text_params: TextParams) -> Self {
        Self {
            was_rendered: false,
            text_specs: text_params.into(),
        }
    }

    fn render_text(renderer_ctx: &mut RenderCtx, target: &NodeTexture, text: &TextSpec) {
        let font_system = &mut renderer_ctx.text_renderer_ctx.lock().unwrap().font_system;
        let swapchain_format = TextureFormat::Rgba8Unorm;
        let mut cache = SwashCache::new();
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
        let mut buffer = Buffer::new(font_system, Metrics::new(text.font_size, text.line_height));

        buffer.set_size(
            font_system,
            target.resolution.width as f32,
            target.resolution.height as f32,
        );

        buffer.set_text(
            font_system,
            &text.content,
            text.attributes.as_attrs(),
            Shaping::Advanced,
        );
        buffer.shape_until_scroll(font_system);

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
                    left: text.text_box.top_left_corner.0 as f32,
                    top: text.text_box.top_left_corner.1 as f32,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: target.resolution.width as i32,
                        bottom: target.resolution.height as i32,
                    },
                    default_color: Color::rgb(255, 255, 255),
                }],
                &mut cache,
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
    }

    pub fn render(&self, ctx: &mut RenderCtx, target: &NodeTexture) {
        if self.was_rendered {
            return;
        }
        Self::render_text(ctx, target, &self.text_specs);
    }
}
