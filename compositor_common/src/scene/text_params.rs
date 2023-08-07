use std::sync::Arc;

use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Align {
    Left,
    Right,
    Justified,
    Center,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub struct TextParams {
    pub content: Arc<str>,
    /// in pixels
    pub font_size: f32,
    /// in pixels, default: same as font_size
    pub line_height: Option<f32>,
    #[serde(default = "default_color")]
    pub color_rgba: (u8, u8, u8, u8),
    /// https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#family-name-value   
    /// use font family name, not generic family name
    #[serde(default = "default_family")]
    pub font_family: String,
    #[serde(default = "default_style")]
    pub style: Style,
    #[serde(default = "default_align")]
    pub align: Align,
    #[serde(default = "default_wrap")]
    pub wrap: Wrap
}

fn default_color() -> (u8, u8, u8, u8) {
    (255, 255, 255, 255)
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