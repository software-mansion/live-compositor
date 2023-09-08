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
    apply_matrix: ApplyTransformationMatrix
}

impl BuiltinTransformations {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, InitBuiltinError> {
        Ok(Self {
            apply_matrix: ApplyTransformationMatrix::new(wgpu_ctx).map_err(InitBuiltinError::ApplyTransformationMatrix)?,
        })
    }

    pub fn shader(&self, transformation: &BuiltinSpec) -> Arc<Shader> {
        match transformation {
            BuiltinSpec::TransformToResolution { strategy, .. } => match strategy {
                TransformToResolutionStrategy::Stretch => self.apply_matrix.0.clone(),
                TransformToResolutionStrategy::Fill => self.apply_matrix.0.clone(),
                TransformToResolutionStrategy::Fit { .. } => self.apply_matrix.0.clone(),
            },
            BuiltinSpec::FixedPositionLayout { .. } => self.apply_matrix.0.clone(),
            BuiltinSpec::Grid { .. } => self.apply_matrix.0.clone(),
        }
    }
}

pub struct ApplyTransformationMatrix(Arc<Shader>);

impl ApplyTransformationMatrix {
    fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, CreateShaderError> {
        Ok(Self(Arc::new(Shader::new(
            wgpu_ctx,
            include_str!("./apply_transformation_matrix.wgsl").into(),
            FallbackStrategy::FallbackIfAllInputsMissing,
        )?)))
    }
}
