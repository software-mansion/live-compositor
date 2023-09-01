use std::sync::Arc;

use compositor_common::scene::{
    builtin_transformations::{BuiltinSpec, TransformToResolutionStrategy},
    Resolution,
};

use crate::{
    renderer::{WgpuCtx, WgpuError},
    transformations::shader::Shader,
    utils::rgba_to_wgpu_color,
};

pub struct BuiltinTransformations {
    transform_resolution: ConvertResolutionTransformations,
    fixed_position_layout: FixedPositionLayout,
}

impl BuiltinTransformations {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, WgpuError> {
        Ok(Self {
            transform_resolution: ConvertResolutionTransformations::new(wgpu_ctx)?,
            fixed_position_layout: FixedPositionLayout::new(wgpu_ctx)?,
        })
    }

    pub fn shader(&self, transformation: &BuiltinSpec) -> Arc<Shader> {
        match transformation {
            BuiltinSpec::TransformToResolution { strategy, .. } => match strategy {
                TransformToResolutionStrategy::Stretch => self.transform_resolution.stretch.clone(),
                TransformToResolutionStrategy::Fill => self.transform_resolution.fill.clone(),
                TransformToResolutionStrategy::Fit { .. } => self.transform_resolution.fit.clone(),
            },
            BuiltinSpec::FixedPositionLayout { .. } => self.fixed_position_layout.0.clone(),
        }
    }

    pub fn output_resolution(
        spec: &BuiltinSpec,
        _input_resolutions: &[Option<Resolution>],
    ) -> Resolution {
        match spec {
            BuiltinSpec::TransformToResolution { resolution, .. } => *resolution,
            BuiltinSpec::FixedPositionLayout { resolution, .. } => *resolution,
        }
    }

    pub fn clear_color(transformation: &BuiltinSpec) -> Option<wgpu::Color> {
        match transformation {
            BuiltinSpec::TransformToResolution { strategy, .. } => match strategy {
                TransformToResolutionStrategy::Fit {
                    background_color_rgba,
                } => Some(rgba_to_wgpu_color(background_color_rgba)),
                TransformToResolutionStrategy::Stretch | TransformToResolutionStrategy::Fill => {
                    None
                }
            },
            BuiltinSpec::FixedPositionLayout {
                background_color_rgba,
                ..
            } => Some(rgba_to_wgpu_color(background_color_rgba)),
        }
    }
}

pub struct ConvertResolutionTransformations {
    stretch: Arc<Shader>,
    fill: Arc<Shader>,
    fit: Arc<Shader>,
}

impl ConvertResolutionTransformations {
    pub(crate) fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, WgpuError> {
        Ok(Self {
            stretch: Arc::new(Shader::new(
                wgpu_ctx,
                include_str!("./transform_to_resolution/stretch.wgsl").into(),
            )?),
            fill: Arc::new(Shader::new(
                wgpu_ctx,
                include_str!("./transform_to_resolution/fill.wgsl").into(),
            )?),
            fit: Arc::new(Shader::new(
                wgpu_ctx,
                include_str!("./transform_to_resolution/fit.wgsl").into(),
            )?),
        })
    }
}

pub struct FixedPositionLayout(Arc<Shader>);

impl FixedPositionLayout {
    fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, WgpuError> {
        Ok(Self(Arc::new(Shader::new(
            wgpu_ctx,
            include_str!("./fixed_position_layout.wgsl").into(),
        )?)))
    }
}
