use std::sync::Arc;

use compositor_common::{
    renderer_spec::FallbackStrategy,
    scene::builtin_transformations::{BuiltinSpec, TransformToResolutionStrategy},
};

use crate::{
    renderer::WgpuCtx,
    transformations::shader::{CreateShaderError, Shader},
};

use super::error::InitBuiltinError;

pub struct BuiltinTransformations {
    transform_resolution: ConvertResolutionTransformations,
    fixed_position_layout: FixedPositionLayout,
    grid: Grid,
}

impl BuiltinTransformations {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, InitBuiltinError> {
        Ok(Self {
            transform_resolution: ConvertResolutionTransformations::new(wgpu_ctx)?,
            fixed_position_layout: FixedPositionLayout::new(wgpu_ctx)
                .map_err(InitBuiltinError::FixedPositionLayout)?,
            grid: Grid::new(wgpu_ctx).map_err(InitBuiltinError::Grid)?,
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
            BuiltinSpec::Grid { .. } => self.grid.0.clone(),
        }
    }
}

pub struct ConvertResolutionTransformations {
    stretch: Arc<Shader>,
    fill: Arc<Shader>,
    fit: Arc<Shader>,
}

impl ConvertResolutionTransformations {
    pub(crate) fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, InitBuiltinError> {
        Ok(Self {
            stretch: Arc::new(
                Shader::new(
                    wgpu_ctx,
                    include_str!("./transform_to_resolution/stretch.wgsl").into(),
                    FallbackStrategy::FallbackIfAllInputsMissing,
                )
                .map_err(InitBuiltinError::StretchToResolution)?,
            ),
            fill: Arc::new(
                Shader::new(
                    wgpu_ctx,
                    include_str!("./transform_to_resolution/fill.wgsl").into(),
                    FallbackStrategy::FallbackIfAllInputsMissing,
                )
                .map_err(InitBuiltinError::FillToResolution)?,
            ),
            fit: Arc::new(
                Shader::new(
                    wgpu_ctx,
                    include_str!("./transform_to_resolution/fit.wgsl").into(),
                    FallbackStrategy::FallbackIfAllInputsMissing,
                )
                .map_err(InitBuiltinError::FitToResolution)?,
            ),
        })
    }
}

pub struct FixedPositionLayout(Arc<Shader>);

impl FixedPositionLayout {
    fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, CreateShaderError> {
        Ok(Self(Arc::new(Shader::new(
            wgpu_ctx,
            include_str!("./fixed_position_layout.wgsl").into(),
            FallbackStrategy::FallbackIfAllInputsMissing,
        )?)))
    }
}

pub struct Grid(Arc<Shader>);

impl Grid {
    fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, CreateShaderError> {
        Ok(Self(Arc::new(Shader::new(
            wgpu_ctx,
            include_str!("./grid.wgsl").into(),
            FallbackStrategy::FallbackIfAllInputsMissing
        )?)))
    }
}
