use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HorizontalAlign {
    Left,
    Right,
    Justified,
    Center,
}

impl From<HorizontalAlign> for glyphon::cosmic_text::Align {
    fn from(align: HorizontalAlign) -> Self {
        match align {
            HorizontalAlign::Left => glyphon::cosmic_text::Align::Left,
            HorizontalAlign::Right => glyphon::cosmic_text::Align::Right,
            HorizontalAlign::Justified => glyphon::cosmic_text::Align::Justified,
            HorizontalAlign::Center => glyphon::cosmic_text::Align::Center,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}
