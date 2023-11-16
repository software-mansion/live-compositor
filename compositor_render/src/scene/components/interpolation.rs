use compositor_common::util::{ContinuousValue, InterpolationState};

use super::{HorizontalPosition, Position, RelativePosition, VerticalPosition};

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
            (Position::Relative(start), Position::Relative(end)) => {
                Position::Relative(ContinuousValue::interpolate(start, end, state))
            }
            (_, end) => *end,
        }
    }
}

impl ContinuousValue for RelativePosition {
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

impl ContinuousValue for VerticalPosition {
    fn interpolate(start: &Self, end: &Self, state: InterpolationState) -> Self {
        match (start, end) {
            (VerticalPosition::Top(start), VerticalPosition::Top(end)) => {
                Self::Top(ContinuousValue::interpolate(start, end, state))
            }
            (VerticalPosition::Bottom(start), VerticalPosition::Bottom(end)) => {
                Self::Bottom(ContinuousValue::interpolate(start, end, state))
            }
            (_, end) => *end,
        }
    }
}

impl ContinuousValue for HorizontalPosition {
    fn interpolate(start: &Self, end: &Self, state: InterpolationState) -> Self {
        match (start, end) {
            (HorizontalPosition::Left(start), HorizontalPosition::Left(end)) => {
                Self::Left(ContinuousValue::interpolate(start, end, state))
            }
            (HorizontalPosition::Right(start), HorizontalPosition::Right(end)) => {
                Self::Right(ContinuousValue::interpolate(start, end, state))
            }
            (_, end) => *end,
        }
    }
}
