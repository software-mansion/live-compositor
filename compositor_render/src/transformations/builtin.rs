use compositor_common::scene::{
    builtin_transformations::{BuiltinSpec, TransformToResolutionStrategy},
    Resolution,
};

use crate::utils::rgba_to_wgpu_color;

pub mod error;
pub mod node;
pub mod params;
pub mod transformations;
pub mod utils;

#[derive(Debug, Clone)]
pub struct Builtin(pub BuiltinSpec);

impl Builtin {
    pub fn clear_color(&self) -> Option<wgpu::Color> {
        match &self.0 {
            BuiltinSpec::TransformToResolution { strategy, .. } => match strategy {
                TransformToResolutionStrategy::Stretch | TransformToResolutionStrategy::Fill => {
                    None
                }
                TransformToResolutionStrategy::Fit {
                    background_color_rgba,
                } => Some(rgba_to_wgpu_color(background_color_rgba)),
            },
            BuiltinSpec::FixedPositionLayout {
                background_color_rgba,
                ..
            } => Some(rgba_to_wgpu_color(background_color_rgba)),
            BuiltinSpec::TiledLayout {
                background_color_rgba,
                ..
            } => Some(rgba_to_wgpu_color(background_color_rgba)),
            BuiltinSpec::CornersRounding { .. } => Some(wgpu::Color::TRANSPARENT),
            BuiltinSpec::MirrorImage { .. } => None,
        }
    }

    pub fn output_resolution(&self, input_resolutions: &[Option<Resolution>]) -> Resolution {
        fn first_input_resolution(input_resolutions: &[Option<Resolution>]) -> Resolution {
            input_resolutions
                .first()
                .copied()
                .flatten()
                .unwrap_or(Resolution {
                    width: 1,
                    height: 1,
                })
        }

        match self.0 {
            BuiltinSpec::TransformToResolution { resolution, .. } => resolution,
            BuiltinSpec::FixedPositionLayout { resolution, .. } => resolution,
            BuiltinSpec::TiledLayout { resolution, .. } => resolution,
            BuiltinSpec::MirrorImage { .. } => first_input_resolution(input_resolutions),
            BuiltinSpec::CornersRounding { .. } => first_input_resolution(input_resolutions),
        }
    }

    pub fn resolution_from_spec(&self) -> Option<Resolution> {
        match self.0 {
            BuiltinSpec::TransformToResolution { resolution, .. } => Some(resolution),
            BuiltinSpec::FixedPositionLayout { resolution, .. } => Some(resolution),
            BuiltinSpec::TiledLayout { resolution, .. } => Some(resolution),
            BuiltinSpec::MirrorImage { .. } => None,
            BuiltinSpec::CornersRounding { .. } => None,
        }
    }
}
