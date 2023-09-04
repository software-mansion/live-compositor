use std::sync::Arc;

use compositor_common::scene::builtin_transformations::{
    BuiltinSpec, TransformToResolutionStrategy,
};

use crate::{
    renderer::WgpuCtx,
    transformations::shader::{CreateShaderError, Shader},
};

#[derive(Debug, thiserror::Error)]
pub enum InitBuiltinError {
    #[error("Failed to initialize fixed_position_layout transformation. {0}")]
    FixedPositionLayout(#[source] CreateShaderError),

    #[error("Failed to initialize fit transform_to_resolution transformation. {0}")]
    FitToResolution(#[source] CreateShaderError),

    #[error("Failed to initialize fill transform_to_resolution transformation. {0}")]
    FillToResolution(#[source] CreateShaderError),

    #[error("Failed to initialize stretch transform_to_resolution transformation. {0}")]
    StretchToResolution(#[source] CreateShaderError),
}

pub struct BuiltinsCollection {
    transform_resolution: ConvertResolutionTransformations,
    fixed_position_layout: FixedPositionLayout,
}

impl BuiltinsCollection {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, InitBuiltinError> {
        Ok(Self {
            transform_resolution: ConvertResolutionTransformations::new(wgpu_ctx)?,
            fixed_position_layout: FixedPositionLayout::new(wgpu_ctx)
                .map_err(InitBuiltinError::FixedPositionLayout)?,
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
    pub(crate) fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, InitBuiltinError> {
        Ok(Self {
            stretch: Arc::new(
                Shader::new(
                    wgpu_ctx,
                    include_str!("./transform_to_resolution/stretch.wgsl").into(),
                )
                .map_err(InitBuiltinError::StretchToResolution)?,
            ),
            fill: Arc::new(
                Shader::new(
                    wgpu_ctx,
                    include_str!("./transform_to_resolution/fill.wgsl").into(),
                )
                .map_err(InitBuiltinError::FillToResolution)?,
            ),
            fit: Arc::new(
                Shader::new(
                    wgpu_ctx,
                    include_str!("./transform_to_resolution/fit.wgsl").into(),
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
        )?)))
    }
}
