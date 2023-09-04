use compositor_common::scene::{
    builtin_transformations::{BuiltinSpec, TransformToResolutionStrategy},
    Resolution,
};

use self::{
    fixed_position_layout::FixedPositionLayoutParams,
    transform_to_resolution::{FillParams, FitParams},
};

use super::Builtin;

mod fixed_position_layout;
mod transform_to_resolution;

pub enum BuiltinParams {
    FixedPositionLayout(FixedPositionLayoutParams),
    Fit(FitParams),
    Fill(FillParams),
    None,
}

impl BuiltinParams {
    pub fn new(spec: &Builtin, input_resolutions: &[Option<Resolution>]) -> Self {
        match &spec.0 {
            BuiltinSpec::TransformToResolution {
                strategy,
                resolution,
            } => {
                let input_resolution = input_resolutions[0];

                Self::new_transform_to_resolution(strategy, input_resolution.as_ref(), *resolution)
            }
            BuiltinSpec::FixedPositionLayout {
                texture_layouts,
                resolution,
                ..
            } => BuiltinParams::FixedPositionLayout(FixedPositionLayoutParams::new(
                texture_layouts,
                input_resolutions,
                *resolution,
            )),
        }
    }

    fn new_transform_to_resolution(
        strategy: &TransformToResolutionStrategy,
        input_resolution: Option<&Resolution>,
        output_resolution: Resolution,
    ) -> Self {
        let Some(input_resolution) = input_resolution else {
            return BuiltinParams::None;
        };

        match strategy {
            TransformToResolutionStrategy::Stretch => BuiltinParams::None,
            TransformToResolutionStrategy::Fill => {
                BuiltinParams::Fill(FillParams::new(*input_resolution, output_resolution))
            }
            TransformToResolutionStrategy::Fit { .. } => {
                BuiltinParams::Fit(FitParams::new(*input_resolution, output_resolution))
            }
        }
    }

    pub fn shader_buffer_content(&self) -> bytes::Bytes {
        match self {
            BuiltinParams::FixedPositionLayout(fixed_position_layout) => {
                fixed_position_layout.shader_buffer_content()
            }
            BuiltinParams::Fit(fit_params) => fit_params.shader_buffer_content(),
            BuiltinParams::Fill(fill_params) => fill_params.shader_buffer_content(),
            BuiltinParams::None => todo!(),
        }
    }
}
