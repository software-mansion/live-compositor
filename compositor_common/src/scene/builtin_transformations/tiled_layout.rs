use crate::util::align::VerticalAlign;
use crate::util::colors::RGBAColor;
use crate::{scene::Resolution, util::align::HorizontalAlign};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TiledLayoutSpec {
    #[serde(default)]
    pub background_color_rgba: RGBAColor,
    #[serde(default = "default_tile_aspect_ratio")]
    pub tile_aspect_ratio: (u32, u32),
    pub resolution: Resolution,

    #[serde(default)]
    /// in pixels
    pub margin: u32,
    #[serde(default)]
    pub padding: u32,
    #[serde(default = "default_horizontal_alignment")]
    pub horizontal_alignment: HorizontalAlign,
    #[serde(default = "default_vertical_alignment")]
    pub vertical_alignment: VerticalAlign,
}

fn default_tile_aspect_ratio() -> (u32, u32) {
    (16, 9)
}

fn default_horizontal_alignment() -> HorizontalAlign {
    HorizontalAlign::Center
}

fn default_vertical_alignment() -> VerticalAlign {
    VerticalAlign::Center
}
