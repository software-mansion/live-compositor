use std::sync::Arc;

use compositor_common::scene::builtin_transformations::{
    BuiltinSpec, TransformToResolutionStrategy,
};

use crate::{
    renderer::WgpuCtx,
    shader_executor::{CreateShaderError, ShaderExecutor},
};

use super::error::InitBuiltinError;

pub struct BuiltinTransformations {
    apply_matrix: ApplyTransformationMatrix,
    mirror_image: MirrorImage,
    corners_rounding: CornersRounding,
}

impl BuiltinTransformations {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, InitBuiltinError> {
        Ok(Self {
            apply_matrix: ApplyTransformationMatrix::new(wgpu_ctx)
                .map_err(InitBuiltinError::ApplyTransformationMatrix)?,
            mirror_image: MirrorImage::new(wgpu_ctx).map_err(InitBuiltinError::MirrorImage)?,
            corners_rounding: CornersRounding::new(wgpu_ctx)
                .map_err(InitBuiltinError::CornersRounding)?,
        })
    }

    pub fn shader(&self, transformation: &BuiltinSpec) -> Arc<ShaderExecutor> {
        match transformation {
            BuiltinSpec::TransformToResolution { strategy, .. } => match strategy {
                TransformToResolutionStrategy::Stretch => self.apply_matrix.0.clone(),
                TransformToResolutionStrategy::Fill => self.apply_matrix.0.clone(),
                TransformToResolutionStrategy::Fit { .. } => self.apply_matrix.0.clone(),
            },
            BuiltinSpec::FixedPositionLayout { .. } => self.apply_matrix.0.clone(),
            BuiltinSpec::TiledLayout { .. } => self.apply_matrix.0.clone(),
            BuiltinSpec::MirrorImage { .. } => self.mirror_image.0.clone(),
            BuiltinSpec::CornersRounding { .. } => self.corners_rounding.0.clone(),
        }
    }
}

pub struct ApplyTransformationMatrix(Arc<ShaderExecutor>);

impl ApplyTransformationMatrix {
    fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, CreateShaderError> {
        Ok(Self(Arc::new(ShaderExecutor::new(
            wgpu_ctx,
            include_str!("./apply_transformation_matrix.wgsl").into(),
        )?)))
    }
}

pub struct MirrorImage(Arc<ShaderExecutor>);

impl MirrorImage {
    fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, CreateShaderError> {
        Ok(Self(Arc::new(ShaderExecutor::new(
            wgpu_ctx,
            include_str!("./mirror_image.wgsl").into(),
        )?)))
    }
}

pub struct CornersRounding(Arc<ShaderExecutor>);

impl CornersRounding {
    fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, CreateShaderError> {
        Ok(Self(Arc::new(ShaderExecutor::new(
            wgpu_ctx,
            include_str!("./corners_rounding.wgsl").into(),
        )?)))
    }
}
