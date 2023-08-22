use std::sync::Arc;

use compositor_common::scene::{
    builtin_transformations::{BuiltinTransformation, TransformToResolution},
    ShaderParam,
};

use crate::{
    renderer::{WgpuCtx, WgpuError},
    transformations::shader::Shader,
};

pub struct BuiltinTransformations {
    transform_resolution: ConvertResolutionTransformations,
}

impl BuiltinTransformations {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, WgpuError> {
        Ok(Self {
            transform_resolution: ConvertResolutionTransformations::new(wgpu_ctx)?,
        })
    }

    pub fn shader(&self, transformation: &BuiltinTransformation) -> Arc<Shader> {
        match transformation {
            BuiltinTransformation::TransformToResolution(TransformToResolution::Stretch) => {
                self.transform_resolution.stretch.clone()
            }
            BuiltinTransformation::TransformToResolution(TransformToResolution::Fill) => {
                self.transform_resolution.fill.clone()
            }
            BuiltinTransformation::TransformToResolution(TransformToResolution::Fit(_)) => {
                self.transform_resolution.fit.clone()
            }
        }
    }

    pub fn params(transformation: &BuiltinTransformation) -> Option<ShaderParam> {
        match transformation {
            BuiltinTransformation::TransformToResolution(_) => None,
        }
    }

    pub fn clear_color(transformation: &BuiltinTransformation) -> Option<wgpu::Color> {
        match transformation {
            BuiltinTransformation::TransformToResolution(TransformToResolution::Fit(color)) => {
                Some(wgpu::Color {
                    r: color.0 as f64 / 255.0,
                    g: color.1 as f64 / 255.0,
                    b: color.2 as f64 / 255.0,
                    a: color.3 as f64 / 255.0,
                })
            }
            BuiltinTransformation::TransformToResolution(_) => None,
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
