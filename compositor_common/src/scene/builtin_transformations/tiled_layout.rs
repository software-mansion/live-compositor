use crate::util::align::VerticalAlign;
use crate::util::colors::RGBAColor;
use crate::{scene::Resolution, util::align::HorizontalAlign};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TiledLayoutSpec {
    pub background_color_rgba: RGBAColor,
    pub tile_aspect_ratio: (u32, u32),
    pub resolution: Resolution,

    /// in pixels
    pub margin: u32,
    pub padding: u32,
    pub horizontal_alignment: HorizontalAlign,
    pub vertical_alignment: VerticalAlign,
}
