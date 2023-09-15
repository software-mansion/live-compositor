use std::sync::Arc;

use compositor_common::{
    renderer_spec::FallbackStrategy,
    scene::{
        builtin_transformations::{BuiltinSpec, TransformToResolutionStrategy},
        Resolution,
    },
};

use crate::{shader_executor::ShaderExecutor, utils::rgba_to_wgpu_color};

mod box_layout;
pub mod error;
pub mod node;
pub mod params;
pub mod transformations;
pub mod utils;

#[derive(Debug)]
pub struct Builtin {
    pub spec: BuiltinSpec,
    pub executor: Arc<ShaderExecutor>,
}

impl Builtin {
    pub fn clear_color(&self) -> Option<wgpu::Color> {
        match &self.spec {
            BuiltinSpec::TransformToResolution { strategy, .. } => match strategy {
                TransformToResolutionStrategy::Stretch | TransformToResolutionStrategy::Fill => {
                    None
                }
                TransformToResolutionStrategy::Fit {
                    background_color_rgba,
                    ..
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

        match self.spec {
            BuiltinSpec::TransformToResolution { resolution, .. } => resolution,
            BuiltinSpec::FixedPositionLayout { resolution, .. } => resolution,
            BuiltinSpec::TiledLayout { resolution, .. } => resolution,
            BuiltinSpec::MirrorImage { .. } => first_input_resolution(input_resolutions),
            BuiltinSpec::CornersRounding { .. } => first_input_resolution(input_resolutions),
        }
    }

    pub fn resolution_from_spec(&self) -> Option<Resolution> {
        match self.spec {
            BuiltinSpec::TransformToResolution { resolution, .. } => Some(resolution),
            BuiltinSpec::FixedPositionLayout { resolution, .. } => Some(resolution),
            BuiltinSpec::TiledLayout { resolution, .. } => Some(resolution),
            BuiltinSpec::MirrorImage { .. } => None,
            BuiltinSpec::CornersRounding { .. } => None,
        }
    }

    pub fn fallback_strategy(&self) -> FallbackStrategy {
        match self.spec {
            BuiltinSpec::TransformToResolution { .. }
            | BuiltinSpec::FixedPositionLayout { .. }
            | BuiltinSpec::TiledLayout { .. }
            | BuiltinSpec::MirrorImage { .. }
            | BuiltinSpec::CornersRounding { .. } => FallbackStrategy::FallbackIfAllInputsMissing,
        }
    }
}
