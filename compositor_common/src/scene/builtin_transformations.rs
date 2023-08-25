use serde::{Deserialize, Serialize};

use crate::util::RGBAColor;

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "transformation", rename_all = "snake_case")]
pub enum BuiltinTransformationSpec {
    TransformToResolution(TransformToResolution),
    FixedPositionLayout {
        textures_specs: Vec<TextureLayout>,
        #[serde(default = "default_layout_background_color")]
        background_color_rgba: RGBAColor,
    },
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(
    tag = "strategy",
    content = "background_color_rgba",
    rename_all = "snake_case"
)]
pub enum TransformToResolution {
    /// Rescales input in both axis to match output resolution
    Stretch,
    /// Scales input preserving aspect ratio and cuts equal parts
    /// from both sides in "sticking out" dimension
    Fill,
    /// Scales input preserving aspect ratio and
    /// fill the rest of the texture with the provided color
    Fit(RGBAColor),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TextureLayout {
    pub top: Coord,
    pub left: Coord,
    #[serde(default)]
    pub rotation: Degree,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Coord {
    Pixel(i32),
    Percent(i32),
}

impl Coord {
    pub fn pixels(&self, max_pixels: u32) -> i32 {
        match self {
            Coord::Pixel(pixels) => *pixels,
            Coord::Percent(percent) => max_pixels as i32 * percent / 100,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Degree(pub i32);

fn default_layout_background_color() -> RGBAColor {
    RGBAColor(0, 0, 0, 0)
}
