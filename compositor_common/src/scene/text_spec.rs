use std::sync::Arc;

use glyphon::AttrsOwned;
use serde::{Deserialize, Serialize};

use crate::util::{align::HorizontalAlign, colors::RGBAColor};

use super::MAX_NODE_RESOLUTION;

fn default_color() -> RGBAColor {
    RGBAColor(255, 255, 255, 255)
}

fn default_family() -> String {
    String::from("Arial")
}

fn default_style() -> Style {
    Style::Normal
}

fn default_align() -> HorizontalAlign {
    HorizontalAlign::Left
}

fn default_wrap() -> Wrap {
    Wrap::None
}

fn default_weight() -> Weight {
    Weight::Normal
}

fn default_max_width() -> u32 {
    MAX_NODE_RESOLUTION.width as u32
}

fn default_max_height() -> u32 {
    MAX_NODE_RESOLUTION.height as u32
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Style {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Weight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    Semibold,
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
            Weight::Semibold => glyphon::Weight::SEMIBOLD,
            Weight::Bold => glyphon::Weight::BOLD,
            Weight::ExtraBold => glyphon::Weight::EXTRA_BOLD,
            Weight::Black => glyphon::Weight::BLACK,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub struct TextSpec {
    pub content: Arc<str>,
    /// in pixels
    pub font_size: f32,
    /// in pixels, default: same as font_size
    pub line_height: Option<f32>,
    #[serde(default = "default_color")]
    pub color_rgba: RGBAColor,
    /// https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#family-name-value   
    /// use font family name, not generic family name
    #[serde(default = "default_family")]
    pub font_family: String,
    #[serde(default = "default_style")]
    pub style: Style,
    #[serde(default = "default_align")]
    pub align: HorizontalAlign,
    #[serde(default = "default_weight")]
    pub weight: Weight,
    #[serde(default = "default_wrap")]
    pub wrap: Wrap,
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextDimensions {
    /// Renders text and "trims" texture to smallest possible size
    Fitted {
        #[serde(default = "default_max_width")]
        max_width: u32,
        #[serde(default = "default_max_height")]
        max_height: u32,
    },
    FittedColumn {
        width: u32,
        #[serde(default = "default_max_height")]
        max_height: u32,
    },
    /// Renders text according to provided spec
    /// and outputs texture with provided fixed size
    Fixed { width: u32, height: u32 },
}
