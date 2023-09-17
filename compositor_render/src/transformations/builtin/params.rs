use compositor_common::scene::{
    builtin_transformations::{BuiltinSpec, MirrorMode, TransformToResolutionStrategy},
    Resolution,
};

use self::{
    corners_rounding::CornersRoundingParams,
    fixed_position_layout::FixedPositionLayoutParams,
    mirror_image::MirrorModeExt,
    tiled_layout::TiledLayoutParams,
    transform_to_resolution::{FillParams, FitParams},
};

use super::Builtin;

mod corners_rounding;
mod fixed_position_layout;
mod mirror_image;
mod tiled_layout;
mod transform_to_resolution;

#[derive(Debug)]
pub enum BuiltinParams {
    FixedPositionLayout(FixedPositionLayoutParams),
    Fit(FitParams),
    Fill(FillParams),
    TiledLayout(TiledLayoutParams),
    MirrorMode(MirrorMode),
    CornersRounding(CornersRoundingParams),
    None,
}

impl BuiltinParams {
    pub fn new(builtin: &Builtin, input_resolutions: &[Option<Resolution>]) -> Self {
        match &builtin.spec {
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
            BuiltinSpec::TiledLayout(tailed_layout_spec) => BuiltinParams::TiledLayout(
                TiledLayoutParams::new(input_resolutions, tailed_layout_spec),
            ),
            BuiltinSpec::MirrorImage { mode } => BuiltinParams::MirrorMode(*mode),
            BuiltinSpec::CornersRounding { border_radius } => BuiltinParams::CornersRounding(
                CornersRoundingParams::new(*border_radius, input_resolutions),
            ),
        }
    }

    fn new_transform_to_resolution(
        strategy: &TransformToResolutionStrategy,
        input_resolution: Option<&Resolution>,
        output_resolution: Resolution,
    ) -> Self {
        match strategy {
            TransformToResolutionStrategy::Stretch => BuiltinParams::None,
            TransformToResolutionStrategy::Fill => match input_resolution {
                Some(input_resolution) => {
                    BuiltinParams::Fill(FillParams::new(*input_resolution, output_resolution))
                }
                None => BuiltinParams::Fill(FillParams::default()),
            },
            TransformToResolutionStrategy::Fit {
                horizontal_alignment: horizontal_align,
                vertical_alignment: vertical_align,
                ..
            } => match input_resolution {
                Some(input_resolution) => BuiltinParams::Fit(FitParams::new(
                    *input_resolution,
                    output_resolution,
                    *horizontal_align,
                    *vertical_align,
                )),
                None => BuiltinParams::Fit(FitParams::default()),
            },
        }
    }

    /// Returned bytes have to match shader memory layout to work properly.
    /// Should produce buffer with the same size for the same inputs count
    /// https://www.w3.org/TR/WGSL/#memory-layouts
    pub fn shader_buffer_content(&self) -> bytes::Bytes {
        match self {
            BuiltinParams::FixedPositionLayout(fixed_position_layout) => {
                fixed_position_layout.shader_buffer_content()
            }
            BuiltinParams::Fit(fit_params) => fit_params.shader_buffer_content(),
            BuiltinParams::Fill(fill_params) => fill_params.shader_buffer_content(),
            BuiltinParams::TiledLayout(tiled_layout_params) => {
                tiled_layout_params.shader_buffer_content()
            }
            BuiltinParams::MirrorMode(mode) => mode.shader_buffer_content(),
            BuiltinParams::CornersRounding(corners_rounding_params) => {
                corners_rounding_params.shader_buffer_content()
            }
            BuiltinParams::None => bytes::Bytes::new(),
        }
    }
}
