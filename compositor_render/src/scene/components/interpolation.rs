use crate::scene::interpolation::{ContinuousValue, InterpolationState};

use super::{AbsolutePosition, Position};

impl ContinuousValue for Position {
    fn interpolate(start: &Self, end: &Self, state: InterpolationState) -> Self {
        match (start, end) {
            (
                Position::Static { width, height },
                Position::Static {
                    width: width_end,
                    height: height_end,
                },
            ) => Self::Static {
                width: ContinuousValue::interpolate(width, width_end, state),
                height: ContinuousValue::interpolate(height, height_end, state),
            },
            (Position::Absolute(start), Position::Absolute(end)) => {
                Position::Absolute(ContinuousValue::interpolate(start, end, state))
            }
            (_, end) => *end,
        }
    }
}

impl ContinuousValue for AbsolutePosition {
    fn interpolate(start: &Self, end: &Self, state: InterpolationState) -> Self {
        Self {
            width: ContinuousValue::interpolate(&start.width, &end.width, state),
            height: ContinuousValue::interpolate(&start.height, &end.height, state),
            position_horizontal: ContinuousValue::interpolate(
                &start.position_horizontal,
                &end.position_horizontal,
                state,
            ),
            position_vertical: ContinuousValue::interpolate(
                &start.position_vertical,
                &end.position_vertical,
                state,
            ),
            rotation_degrees: ContinuousValue::interpolate(
                &start.rotation_degrees,
                &end.rotation_degrees,
                state,
            ),
        }
    }
}
