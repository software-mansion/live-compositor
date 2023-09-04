use std::sync::Arc;

use compositor_common::scene::{
    builtin_transformations::{BuiltinTransformationSpec, TransformToResolution},
    Resolution, ShaderParam,
};

use crate::{
    renderer::WgpuCtx,
    transformations::shader::{CreateShaderError, Shader},
    utils::rgba_to_wgpu_color,
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

pub struct BuiltinTransformations {
    transform_resolution: ConvertResolutionTransformations,
    fixed_position_layout: FixedPositionLayout,
}

impl BuiltinTransformations {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Result<Self, InitBuiltinError> {
        Ok(Self {
            transform_resolution: ConvertResolutionTransformations::new(wgpu_ctx)?,
            fixed_position_layout: FixedPositionLayout::new(wgpu_ctx)
                .map_err(InitBuiltinError::FixedPositionLayout)?,
        })
    }

    pub fn shader(&self, transformation: &BuiltinTransformationSpec) -> Arc<Shader> {
        match transformation {
            BuiltinTransformationSpec::TransformToResolution(TransformToResolution::Stretch) => {
                self.transform_resolution.stretch.clone()
            }
            BuiltinTransformationSpec::TransformToResolution(TransformToResolution::Fill) => {
                self.transform_resolution.fill.clone()
            }
            BuiltinTransformationSpec::TransformToResolution(TransformToResolution::Fit {
                ..
            }) => self.transform_resolution.fit.clone(),
            BuiltinTransformationSpec::FixedPositionLayout { .. } => {
                self.fixed_position_layout.0.clone()
            }
        }
    }

    pub fn params(
        transformation: &BuiltinTransformationSpec,
        output_resolution: &Resolution,
    ) -> Option<ShaderParam> {
        match transformation {
            BuiltinTransformationSpec::TransformToResolution(_) => None,
            BuiltinTransformationSpec::FixedPositionLayout {
                texture_layouts: textures_layouts,
                ..
            } => {
                let width = output_resolution.width as u32;
                let height = output_resolution.height as u32;

                let layouts: Vec<ShaderParam> = textures_layouts
                    .iter()
                    .map(|layout| {
                        ShaderParam::Struct(vec![
                            ("top", ShaderParam::I32(layout.top.pixels(height))).into(),
                            ("left", ShaderParam::I32(layout.left.pixels(width))).into(),
                            ("rotation", ShaderParam::I32(layout.rotation.0)).into(),
                            ("_padding", ShaderParam::I32(0)).into(),
                        ])
                    })
                    .collect();

                Some(ShaderParam::List(layouts))
            }
        }
    }

    pub fn clear_color(transformation: &BuiltinTransformationSpec) -> Option<wgpu::Color> {
        match transformation {
            BuiltinTransformationSpec::TransformToResolution(TransformToResolution::Fit {
                background_color_rgba,
            }) => Some(rgba_to_wgpu_color(background_color_rgba)),
            BuiltinTransformationSpec::TransformToResolution(_) => None,
            BuiltinTransformationSpec::FixedPositionLayout {
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
