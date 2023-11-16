#[derive(Debug, Clone, Copy)]
pub struct InterpolationState(pub f64);

pub trait ContinuousValue {
    fn interpolate(start: &Self, end: &Self, state: InterpolationState) -> Self;
}

fn interpolate_f64(start: f64, end: f64, state: InterpolationState) -> f64 {
    start + ((end - start) * f64::from(state))
}

impl ContinuousValue for i32 {
    fn interpolate(start: &Self, end: &Self, state: InterpolationState) -> Self {
        interpolate_f64(*start as f64, *end as f64, state) as Self
    }
}

impl ContinuousValue for f64 {
    fn interpolate(start: &Self, end: &Self, state: InterpolationState) -> Self {
        interpolate_f64(*start, *end, state)
    }
}

impl ContinuousValue for f32 {
    fn interpolate(start: &Self, end: &Self, state: InterpolationState) -> Self {
        interpolate_f64(*start as f64, *end as f64, state) as f32
    }
}

impl ContinuousValue for usize {
    fn interpolate(start: &Self, end: &Self, state: InterpolationState) -> Self {
        interpolate_f64(*start as f64, *end as f64, state) as usize
    }
}

impl<T: ContinuousValue + Clone> ContinuousValue for Option<T> {
    fn interpolate(start: &Self, end: &Self, state: InterpolationState) -> Self {
        match (start, end) {
            (Some(start), Some(end)) => Some(ContinuousValue::interpolate(start, end, state)),
            (_, end) => end.clone(),
        }
    }
}

impl From<InterpolationState> for f64 {
    fn from(value: InterpolationState) -> Self {
        value.0
    }
}
