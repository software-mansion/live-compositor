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
    /// default: white (255, 255, 255, 255)
    pub color_rgba: Option<(u8, u8, u8, u8)>,
    /// https://www.w3.org/TR/2018/REC-css-fonts-3-20180920/#family-name-value   
    /// use font family name, not generic family name
    pub font_family: Option<String>,
    /// in pixels, default: same as font_size
    pub line_height: Option<f32>,
    /// default: Normal
    pub style: Option<Style>,
    /// default: Left
    pub align: Option<Align>,
    /// default: None
    pub wrap: Option<Wrap>,
}
