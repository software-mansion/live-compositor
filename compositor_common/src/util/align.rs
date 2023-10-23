#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
    Justified,
}
