use std::{
    cmp::max,
    fmt,
    sync::{Arc, Mutex},
};

use glyphon::{
    AttrsOwned, Buffer, Color, FontSystem, Metrics, Shaping, SwashCache, TextArea, TextAtlas,
    TextBounds,
};
use wgpu::{
    CommandEncoderDescriptor, LoadOp, MultisampleState, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, TextureFormat,
};

use crate::{
    renderer::RenderCtx,
    scene::{
        HorizontalAlign, RGBAColor, TextComponent, TextDimensions, TextStyle, TextWeight, TextWrap,
    },
    utils::rgba_to_wgpu_color,
    wgpu::texture::NodeTexture,
    Resolution,
};

#[derive(Debug, Clone)]
pub(crate) struct TextRenderParams {
    pub(crate) buffer: TextBuffer,
    pub(crate) resolution: Resolution,
    pub(crate) background_color: RGBAColor,
}

#[derive(Clone)]
pub(crate) struct TextBuffer(Arc<glyphon::Buffer>);

impl fmt::Debug for TextBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(glyphon::Buffer))")
    }
}

pub(crate) struct TextRendererNode {
    buffer: TextBuffer,
    resolution: Resolution,
    background_color: wgpu::Color,
    was_rendered: bool,
}

impl TextRendererNode {
    pub(crate) fn new(params: TextRenderParams) -> Self {
        let background_color = rgba_to_wgpu_color(&params.background_color);

        Self {
            buffer: params.buffer,
            resolution: params.resolution,
            background_color,
            was_rendered: false,
        }
    }

    pub(crate) fn render(&mut self, renderer_ctx: &mut RenderCtx, target: &mut NodeTexture) {
        if self.was_rendered {
            return;
        }

        let text_renderer = renderer_ctx.text_renderer_ctx;
        let font_system = &mut text_renderer.font_system.lock().unwrap();
        let cache = &mut text_renderer.swash_cache.lock().unwrap();

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

        text_renderer
            .prepare(
                &renderer_ctx.wgpu_ctx.device,
                &renderer_ctx.wgpu_ctx.queue,
                font_system,
                &mut atlas,
                glyphon::Resolution {
                    width: self.resolution.width as u32,
                    height: self.resolution.height as u32,
                },
                [TextArea {
                    buffer: &self.buffer.0,
                    left: 0 as f32,
                    top: 0 as f32,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: self.resolution.width as i32,
                        bottom: self.resolution.height as i32,
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

        let target_state = target.ensure_size(renderer_ctx.wgpu_ctx, self.resolution);
        let view = &target_state.rgba_texture().texture().view;
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.background_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            text_renderer.render(&atlas, &mut pass).unwrap();
        }

        renderer_ctx.wgpu_ctx.queue.submit(Some(encoder.finish()));
        self.was_rendered = true;
    }
}

pub(crate) struct TextParams {
    content: Arc<str>,
    attributes: AttrsOwned,
    font_size: f32,
    line_height: f32,
    align: glyphon::cosmic_text::Align,
    wrap: glyphon::cosmic_text::Wrap,
}

impl From<&TextComponent> for TextParams {
    fn from(text: &TextComponent) -> Self {
        let RGBAColor(r, g, b, a) = text.color;
        let color = glyphon::Color::rgba(r, g, b, a);

        let family = glyphon::FamilyOwned::Name(text.font_family.to_string());

        let style = match text.style {
            TextStyle::Normal => glyphon::Style::Normal,
            TextStyle::Italic => glyphon::Style::Italic,
            TextStyle::Oblique => glyphon::Style::Oblique,
        };
        let weight = match text.weight {
            TextWeight::Thin => glyphon::Weight::THIN,
            TextWeight::ExtraLight => glyphon::Weight::EXTRA_LIGHT,
            TextWeight::Light => glyphon::Weight::LIGHT,
            TextWeight::Normal => glyphon::Weight::NORMAL,
            TextWeight::Medium => glyphon::Weight::MEDIUM,
            TextWeight::SemiBold => glyphon::Weight::SEMIBOLD,
            TextWeight::Bold => glyphon::Weight::BOLD,
            TextWeight::ExtraBold => glyphon::Weight::EXTRA_BOLD,
            TextWeight::Black => glyphon::Weight::BLACK,
        };
        let wrap = match text.wrap {
            TextWrap::None => glyphon::cosmic_text::Wrap::None,
            TextWrap::Glyph => glyphon::cosmic_text::Wrap::Glyph,
            TextWrap::Word => glyphon::cosmic_text::Wrap::Word,
        };
        let align = match text.align {
            HorizontalAlign::Left => glyphon::cosmic_text::Align::Left,
            HorizontalAlign::Right => glyphon::cosmic_text::Align::Right,
            HorizontalAlign::Justified => glyphon::cosmic_text::Align::Justified,
            HorizontalAlign::Center => glyphon::cosmic_text::Align::Center,
        };

        Self {
            attributes: glyphon::AttrsOwned {
                color_opt: Some(color),
                family_owned: family,
                stretch: Default::default(),
                style,
                weight,
                metadata: Default::default(),
            },
            content: text.text.clone(),
            font_size: text.font_size,
            line_height: text.line_height,
            align,
            wrap,
        }
    }
}

pub(crate) struct TextRendererCtx {
    font_system: Mutex<FontSystem>,
    swash_cache: Mutex<SwashCache>,
}

impl TextRendererCtx {
    pub(crate) fn new() -> Self {
        Self {
            font_system: Mutex::new(FontSystem::new()),
            swash_cache: Mutex::new(SwashCache::new()),
        }
    }
}

impl TextRendererCtx {
    pub(crate) fn layout_text(
        &self,
        text_params: TextParams,
        text_resolution: TextDimensions,
    ) -> (TextBuffer, Resolution) {
        let font_system = &mut self.font_system.lock().unwrap();
        let mut buffer = Buffer::new(
            font_system,
            Metrics::new(text_params.font_size, text_params.line_height),
        );

        buffer.set_text(
            font_system,
            &text_params.content,
            text_params.attributes.as_attrs(),
            Shaping::Advanced,
        );
        buffer.set_wrap(font_system, text_params.wrap);

        let texture_size = match text_resolution {
            TextDimensions::Fixed { width, height } => Resolution {
                width: width as usize,
                height: height as usize,
            },
            TextDimensions::Fitted {
                max_width,
                max_height,
            } => {
                buffer.set_size(font_system, max_width, max_height);
                buffer.shape_until_scroll(font_system);
                Self::get_text_resolution(
                    buffer.lines.iter(),
                    text_params.line_height,
                    text_params.font_size,
                )
            }
            TextDimensions::FittedColumn { width, max_height } => {
                buffer.set_size(font_system, width, max_height);
                buffer.shape_until_scroll(font_system);
                let text_size = Self::get_text_resolution(
                    buffer.lines.iter(),
                    text_params.line_height,
                    text_params.font_size,
                );

                Resolution {
                    width: width as usize,
                    height: text_size.height,
                }
            }
        };

        buffer.set_size(
            font_system,
            texture_size.width as f32,
            texture_size.height as f32 + text_params.line_height,
        );
        for line in &mut buffer.lines {
            line.set_align(Some(text_params.align));
        }
        buffer.shape_until_scroll(font_system);

        (TextBuffer(buffer.into()), texture_size)
    }

    fn get_text_resolution<'a, I: Iterator<Item = &'a glyphon::BufferLine>>(
        lines: I,
        line_height: f32,
        font_size: f32,
    ) -> Resolution {
        let mut width = 0;
        let mut lines_count = 0u32;

        for line in lines {
            if let Some(layout) = line.layout_opt() {
                for layout_line in layout {
                    lines_count += 1;
                    width = max(width, layout_line.w.ceil() as usize);
                }
            }
        }

        let last_line_padding = font_size / 5.0;
        let height = (lines_count as f32 * line_height.ceil() + last_line_padding) as usize;
        Resolution { width, height }
    }
}
