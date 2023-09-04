use compositor_common::scene::{
    builtin_transformations::{BuiltinSpec, TransformToResolutionStrategy},
    Resolution,
};

use crate::utils::rgba_to_wgpu_color;

pub mod collection;
pub mod node;
pub mod params;

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
        }
    }

    pub fn output_resolution(&self, _input_resolutions: &[Option<Resolution>]) -> Resolution {
        match self.0 {
            BuiltinSpec::TransformToResolution { resolution, .. } => resolution,
            BuiltinSpec::FixedPositionLayout { resolution, .. } => resolution,
        }
    }

    pub fn resolution_from_spec(&self) -> Option<Resolution> {
        match self.0 {
            BuiltinSpec::TransformToResolution { resolution, .. } => Some(resolution),
            BuiltinSpec::FixedPositionLayout { resolution, .. } => Some(resolution),
        }
    }
}
