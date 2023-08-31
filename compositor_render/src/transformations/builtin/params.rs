use compositor_common::scene::{
    builtin_transformations::{BuiltinTransformationSpec, TransformToResolutionStrategy},
    Resolution,
};

use crate::utils::rgba_to_wgpu_color;

use self::{
    fixed_position_layout::FixedPositionLayoutParams,
    transform_to_resolution::{TransformToResolutionFillParams, TransformToResolutionFitParams},
};

pub enum BuiltinTransformationParams {
    FixedPositionLayout(FixedPositionLayoutParams),
    TransformToResolutionFit(TransformToResolutionFitParams),
    TransformToResolutionFill(TransformToResolutionFillParams),
    None,
}

mod fixed_position_layout;
mod transform_to_resolution;

impl BuiltinTransformationParams {
    pub fn new(
        spec: BuiltinTransformationSpec,
        input_resolutions: Vec<Option<Resolution>>,
        output_resolution: Resolution,
    ) -> Self {
        match spec {
            BuiltinTransformationSpec::TransformToResolution { strategy } => {
                let Some(input_resolution) = input_resolutions.get(0) else { 
                    return BuiltinTransformationParams::None;
                };

                Self::new_transform_to_resolution(strategy, input_resolution, output_resolution)
            }
            BuiltinTransformationSpec::FixedPositionLayout {
                texture_layouts,
                background_color_rgba,
            } => todo!(),
        }
    }

    fn new_transform_to_resolution(
        strategy: TransformToResolutionStrategy,
        input_resolution: &Option<Resolution>,
        output_resolution: Resolution,
    ) -> Self {
        let Some(input_resolution) = input_resolution else {
            return BuiltinTransformationParams::None;
        };

        match strategy {
            TransformToResolutionStrategy::Stretch => BuiltinTransformationParams::None,
            TransformToResolutionStrategy::Fill => {
                BuiltinTransformationParams::TransformToResolutionFill(
                    TransformToResolutionFillParams::new(*input_resolution, output_resolution),
                )
            }
            TransformToResolutionStrategy::Fit {
                background_color_rgba,
            } => BuiltinTransformationParams::TransformToResolutionFit(
                TransformToResolutionFitParams::new(
                    *input_resolution,
                    output_resolution,
                    rgba_to_wgpu_color(&background_color_rgba),
                ),
            ),
        }
    }
}
