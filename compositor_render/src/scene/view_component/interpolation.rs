use crate::scene::types::interpolation::{ContinuousValue, InterpolationState};

use super::ViewComponentParam;

impl ContinuousValue for ViewComponentParam {
    fn interpolate(start: &Self, end: &Self, state: InterpolationState) -> Self {
        Self {
            id: end.id.clone(),
            direction: end.direction.clone(),
            position: ContinuousValue::interpolate(&start.position, &end.position, state),
            background_color: end.background_color,
            overflow: end.overflow,
            border_radius: end.border_radius,
            border_width: end.border_width,
            border_color: end.border_color, // TODO(wkozyra95)
        }
    }
}
