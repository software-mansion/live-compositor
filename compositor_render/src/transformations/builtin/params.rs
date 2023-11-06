use compositor_common::{
    scene::{
        builtin_transformations::{BuiltinSpec, FitToResolutionSpec, MirrorMode},
        Resolution,
    },
    util::{ContinuousValue, InterpolationState},
};
use nalgebra_glm::Mat4;

use self::{
    box_layout_params::BoxLayoutParams,
    corners_rounding::CornersRoundingParams,
    fixed_position_layout::new_fixed_position_layout_params,
    mirror_image::MirrorModeExt,
    tiled_layout::new_tiled_layout_params,
    transform_to_resolution::{new_fit_to_resolution_params, FillParams},
};

use super::{box_layout::BoxLayout, utils::mat4_to_bytes, BuiltinState, BuiltinTransition};

mod box_layout_params;
mod corners_rounding;
mod fixed_position_layout;
mod mirror_image;
mod tiled_layout;
mod transform_to_resolution;

#[derive(Debug, Clone)]
pub(super) enum RenderParams {
    BoxLayout(BoxLayoutParams),
    Fill(FillParams),
    MirrorMode(MirrorMode),
    CornersRounding(CornersRoundingParams),
    None,
}

impl RenderParams {
    pub fn new(state: &BuiltinState, input_resolutions: &[Option<Resolution>]) -> Self {
        match state {
            BuiltinState::Interpolated { transition, state } => {
                Self::new_from_transition(transition, input_resolutions, *state)
            }
            BuiltinState::Static(spec) => Self::new_from_spec(spec, input_resolutions),
        }
    }

    fn new_from_transition(
        transition: &BuiltinTransition,
        input_resolutions: &[Option<Resolution>],
        state: InterpolationState,
    ) -> Self {
        match transition {
            BuiltinTransition::FixedPositionLayout(start, end) => Self::interpolate(
                &Self::new_from_spec(
                    &BuiltinSpec::FixedPositionLayout(start.clone()),
                    input_resolutions,
                ),
                &Self::new_from_spec(
                    &BuiltinSpec::FixedPositionLayout(end.clone()),
                    input_resolutions,
                ),
                state,
            ),
        }
    }

    fn new_from_spec(spec: &BuiltinSpec, input_resolutions: &[Option<Resolution>]) -> Self {
        match spec {
            BuiltinSpec::FixedPositionLayout(spec) => {
                RenderParams::BoxLayout(new_fixed_position_layout_params(spec, input_resolutions))
            }
            BuiltinSpec::TiledLayout(spec) => {
                RenderParams::BoxLayout(new_tiled_layout_params(spec, input_resolutions))
            }
            BuiltinSpec::MirrorImage { mode } => RenderParams::MirrorMode(*mode),
            BuiltinSpec::CornersRounding { border_radius } => RenderParams::CornersRounding(
                CornersRoundingParams::new(*border_radius, input_resolutions),
            ),
            BuiltinSpec::FitToResolution(FitToResolutionSpec {
                background_color_rgba: _,
                horizontal_alignment,
                vertical_alignment,
                resolution,
            }) => match input_resolutions.get(0).unwrap_or(&None) {
                Some(input_resolution) => RenderParams::BoxLayout(new_fit_to_resolution_params(
                    *input_resolution,
                    *resolution,
                    *horizontal_alignment,
                    *vertical_alignment,
                )),
                None => RenderParams::BoxLayout(BoxLayoutParams {
                    boxes: vec![BoxLayout::NONE],
                    output_resolution: *resolution,
                }),
            },
            BuiltinSpec::FillToResolution { resolution } => {
                match input_resolutions.get(0).unwrap_or(&None) {
                    Some(input_resolution) => {
                        RenderParams::Fill(FillParams::new(*input_resolution, *resolution))
                    }
                    None => RenderParams::Fill(FillParams::default()),
                }
            }
            BuiltinSpec::StretchToResolution { .. } => RenderParams::None,
        }
    }

    /// Returned bytes have to match shader memory layout to work properly.
    /// Should produce buffer with the same size for the same inputs count
    /// https://www.w3.org/TR/WGSL/#memory-layouts
    pub fn shader_buffer_content(&self) -> bytes::Bytes {
        match self {
            RenderParams::BoxLayout(layout) => layout.shader_buffer_content(),
            RenderParams::MirrorMode(mode) => mode.shader_buffer_content(),
            RenderParams::CornersRounding(corners_rounding_params) => {
                corners_rounding_params.shader_buffer_content()
            }
            RenderParams::Fill(fill_params) => fill_params.shader_buffer_content(),
            RenderParams::None => mat4_to_bytes(&Mat4::identity()),
        }
    }
}

impl ContinuousValue for RenderParams {
    fn interpolate(start: &Self, end: &Self, state: InterpolationState) -> Self {
        match (start, end) {
            (RenderParams::BoxLayout(start), RenderParams::BoxLayout(end)) => {
                RenderParams::BoxLayout(BoxLayoutParams::interpolate(start, end, state))
            }
            (start, _) => start.clone(),
        }
    }
}
