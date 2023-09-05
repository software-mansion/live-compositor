use compositor_common::{renderer_spec::FallbackStrategy, util::RGBAColor};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InputState {
    Filled,
    Empty,
}

impl InputState {
    pub fn is_empty(&self) -> bool {
        match self {
            InputState::Filled => false,
            InputState::Empty => true,
        }
    }
}

pub(crate) fn rgba_to_wgpu_color(rgba_color: &RGBAColor) -> wgpu::Color {
    wgpu::Color {
        r: rgba_color.0 as f64 / 255.0,
        g: rgba_color.1 as f64 / 255.0,
        b: rgba_color.2 as f64 / 255.0,
        a: rgba_color.3 as f64 / 255.0,
    }
}

pub(crate) fn does_fallback(
    fallback_strategy: &FallbackStrategy,
    input_states: &[InputState],
) -> bool {
    match fallback_strategy {
        FallbackStrategy::NeverFallback => false,
        FallbackStrategy::FallbackIfAllInputsMissing => {
            input_states.iter().all(|state| state.is_empty())
        }
        FallbackStrategy::FallbackIfAnyInputsMissing => input_states
            .iter()
            .any(|input_state| input_state.is_empty()),
        FallbackStrategy::FallbackIfOnlyInputMissing => {
            input_states.len() == 1
                && input_states
                    .get(0)
                    .map_or(false, |input_state| input_state.is_empty())
        }
    }
}
