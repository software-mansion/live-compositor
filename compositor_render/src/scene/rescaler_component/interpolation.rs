use compositor_common::util::{ContinuousValue, InterpolationState};

use super::RescalerComponentParam;

impl ContinuousValue for RescalerComponentParam {
    fn interpolate(start: &Self, end: &Self, state: InterpolationState) -> Self {
        Self {
            id: end.id.clone(),
            position: ContinuousValue::interpolate(&start.position, &end.position, state),
            mode: end.mode,
            horizontal_align: end.horizontal_align,
            vertical_align: end.vertical_align,
        }
    }
}
