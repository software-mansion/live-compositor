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
                } => Some(background_color_rgba),
            },
            BuiltinSpec::FixedPositionLayout {
                background_color_rgba,
                ..
            } => Some(background_color_rgba),
            BuiltinSpec::Grid {
                background_color_rgba,
                ..
            } => Some(background_color_rgba),
        }
        .map(rgba_to_wgpu_color)
    }

    pub fn output_resolution(&self, _input_resolutions: &[Option<Resolution>]) -> Resolution {
        match self.0 {
            BuiltinSpec::TransformToResolution { resolution, .. } => resolution,
            BuiltinSpec::FixedPositionLayout { resolution, .. } => resolution,
            BuiltinSpec::Grid { resolution, .. } => resolution,
        }
    }

    pub fn resolution_from_spec(&self) -> Option<Resolution> {
        match self.0 {
            BuiltinSpec::TransformToResolution { resolution, .. } => Some(resolution),
            BuiltinSpec::FixedPositionLayout { resolution, .. } => Some(resolution),
            BuiltinSpec::Grid { resolution, .. } => Some(resolution),
        }
    }
}
