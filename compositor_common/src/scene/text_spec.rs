use std::sync::Arc;

use glyphon::AttrsOwned;

use crate::util::{align::HorizontalAlign, colors::RGBAColor};

#[derive(Debug, Clone)]
pub enum Style {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone)]
pub enum Wrap {
    None,
    Glyph,
    Word,
}

impl From<Wrap> for glyphon::cosmic_text::Wrap {
    fn from(wrap: Wrap) -> Self {
        match wrap {
            Wrap::None => glyphon::cosmic_text::Wrap::None,
            Wrap::Glyph => glyphon::cosmic_text::Wrap::Glyph,
            Wrap::Word => glyphon::cosmic_text::Wrap::Word,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Weight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

impl From<&Weight> for glyphon::Weight {
    fn from(value: &Weight) -> Self {
        match value {
            Weight::Thin => glyphon::Weight::THIN,
            Weight::ExtraLight => glyphon::Weight::EXTRA_LIGHT,
            Weight::Light => glyphon::Weight::LIGHT,
            Weight::Normal => glyphon::Weight::NORMAL,
            Weight::Medium => glyphon::Weight::MEDIUM,
            Weight::SemiBold => glyphon::Weight::SEMIBOLD,
            Weight::Bold => glyphon::Weight::BOLD,
            Weight::ExtraBold => glyphon::Weight::EXTRA_BOLD,
            Weight::Black => glyphon::Weight::BLACK,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextSpec {
    pub content: Arc<str>,
    /// in pixels
    pub font_size: f32,
    /// in pixels, default: same as font_size
    pub line_height: Option<f32>,
    pub color_rgba: RGBAColor,
    /// https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#family-name-value   
    /// use font family name, not generic family name
    pub font_family: String,
    pub style: Style,
    pub align: HorizontalAlign,
    pub weight: Weight,
    pub wrap: Wrap,
    pub background_color_rgba: RGBAColor,
    pub dimensions: TextDimensions,
}

impl From<&TextSpec> for AttrsOwned {
    fn from(text_params: &TextSpec) -> Self {
        let RGBAColor(r, g, b, a) = text_params.color_rgba;
        let color = glyphon::Color::rgba(r, g, b, a);

        let family = glyphon::FamilyOwned::Name(text_params.font_family.clone());

        let style = match text_params.style {
            Style::Normal => glyphon::Style::Normal,
            Style::Italic => glyphon::Style::Italic,
            Style::Oblique => glyphon::Style::Oblique,
        };

        AttrsOwned {
            color_opt: Some(color),
            family_owned: family,
            stretch: Default::default(),
            style,
            weight: (&text_params.weight).into(),
            metadata: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TextDimensions {
    /// Renders text and "trims" texture to smallest possible size
    Fitted {
        max_width: u32,
        max_height: u32,
    },
    FittedColumn {
        width: u32,
        max_height: u32,
    },
    /// Renders text according to provided spec
    /// and outputs texture with provided fixed size
    Fixed {
        width: u32,
        height: u32,
    },
}
