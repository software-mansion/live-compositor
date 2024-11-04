use crate::scene::AbsolutePosition;

use super::Position;

impl Position {
    pub(crate) fn with_border(self, border_width: f32) -> Self {
        match self {
            Position::Static { width, height } => Self::Static {
                width: width.map(|w| w + 2.0 * border_width),
                height: height.map(|h| h + 2.0 * border_width),
            },
            Position::Absolute(AbsolutePosition {
                width,
                height,
                position_horizontal,
                position_vertical,
                rotation_degrees,
            }) => Self::Absolute(AbsolutePosition {
                width: width.map(|w| w + 2.0 * border_width),
                height: height.map(|h| h + 2.0 * border_width),
                position_horizontal,
                position_vertical,
                rotation_degrees,
            }),
        }
    }
}
