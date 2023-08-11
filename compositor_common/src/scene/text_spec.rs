use std::sync::Arc;

use glyphon::AttrsOwned;
use serde::{Deserialize, Serialize};

use super::Resolution;

use crate::util::RGBAColor;

fn default_color() -> RGBAColor {
    RGBAColor(255, 255, 255, 255)
}

fn default_family() -> String {
    String::from("Verdana")
}

fn default_style() -> Style {
    Style::Normal
}

fn default_align() -> Align {
    Align::Left
}

fn default_wrap() -> Wrap {
    Wrap::None
}

fn default_max_height() -> u32 {
    4320
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Style {
    Normal,
    Italic,
    Oblique,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Align {
    Left,
    Right,
    Justified,
    Center,
}

impl From<Align> for glyphon::cosmic_text::Align {
    fn from(align: Align) -> Self {
        match align {
            Align::Left => glyphon::cosmic_text::Align::Left,
            Align::Right => glyphon::cosmic_text::Align::Right,
            Align::Justified => glyphon::cosmic_text::Align::Justified,
            Align::Center => glyphon::cosmic_text::Align::Center,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
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
    pub align: Align,
    #[serde(default = "default_wrap")]
    pub wrap: Wrap,
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
            weight: Default::default(),
            metadata: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextResolution {
    /// Renders text and "trims" texture to smallest possible size
    Fitted {
        /// Must be specified with wrap
        max_width: Option<u32>,
        #[serde(default = "default_max_height")]
        max_height: u32,
    },
    /// Renders text according to provided spec
    /// and outputs texture with provided fixed size
    Fixed { resolution: Resolution },
}
