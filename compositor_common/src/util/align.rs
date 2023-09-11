use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
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
