use std::sync::Arc;

use compositor_common::scene::{
    builtin_transformations::{BuiltinTransformation, TextureLayout, TransformToResolution},
    Resolution, ShaderParam,
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
            BuiltinTransformation::FixedPositionLayout { .. } => {
                self.fixed_position_layout.0.clone()
            }
        }
    }

    pub fn params(
        transformation: &BuiltinTransformation,
        output_resolution: &Resolution,
    ) -> Option<ShaderParam> {
        match transformation {
            BuiltinTransformation::TransformToResolution(_) => None,
            BuiltinTransformation::FixedPositionLayout { textures_specs, .. } => {
                let mut layouts = Vec::new();

                for TextureLayout {
                    top,
                    left,
                    rotation,
                } in textures_specs
                {
                    layouts.push(ShaderParam::I32(
                        top.pixels(output_resolution.height as u32),
                    ));
                    layouts.push(ShaderParam::I32(
                        left.pixels(output_resolution.width as u32),
                    ));
                    layouts.push(ShaderParam::I32(rotation.0));
                    layouts.push(ShaderParam::I32(0)); // padding
                }
                Some(ShaderParam::List(layouts))
            }
        }
    }

    pub fn clear_color(transformation: &BuiltinTransformation) -> Option<wgpu::Color> {
        match transformation {
            BuiltinTransformation::TransformToResolution(TransformToResolution::Fit(
                background_color_rgba,
            )) => Some(rgba_to_wgpu_color(background_color_rgba)),
            BuiltinTransformation::TransformToResolution(_) => None,
            BuiltinTransformation::FixedPositionLayout {
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
