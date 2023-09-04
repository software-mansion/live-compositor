use std::sync::Arc;

use compositor_common::scene::builtin_transformations::{
    BuiltinSpec, TransformToResolutionStrategy,
};

use crate::{
    renderer::{WgpuCtx, WgpuError},
    transformations::shader::Shader,
};

pub struct BuiltinsContainer {
    transform_resolution: ConvertResolutionTransformations,
    fixed_position_layout: FixedPositionLayout,
}

impl BuiltinsContainer {
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
