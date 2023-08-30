use std::sync::Arc;

use compositor_common::scene::{
    builtin_transformations::{BuiltinTransformationSpec, TextureLayout, TransformToResolution},
    Resolution, ShaderParam, ShaderParamStructField,
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

    pub fn shader(&self, transformation: &BuiltinTransformationSpec) -> Arc<Shader> {
        match transformation {
            BuiltinTransformationSpec::TransformToResolution(TransformToResolution::Stretch) => {
                self.transform_resolution.stretch.clone()
            }
            BuiltinTransformationSpec::TransformToResolution(TransformToResolution::Fill) => {
                self.transform_resolution.fill.clone()
            }
            BuiltinTransformationSpec::TransformToResolution(TransformToResolution::Fit(_)) => {
                self.transform_resolution.fit.clone()
            }
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
                fn shader_texture_layout(
                    texture_layout: &TextureLayout,
                    output_resolution: &Resolution,
                ) -> ShaderParam {
                    let top = ShaderParamStructField {
                        field_name: "top".into(),
                        value: ShaderParam::I32(
                            texture_layout.top.pixels(output_resolution.height as u32),
                        ),
                    };

                    let left = ShaderParamStructField {
                        field_name: "left".into(),
                        value: ShaderParam::I32(
                            texture_layout.left.pixels(output_resolution.width as u32),
                        ),
                    };

                    let rotation = ShaderParamStructField {
                        field_name: "rotation".into(),
                        value: ShaderParam::I32(texture_layout.rotation.0),
                    };

                    let padding = ShaderParamStructField {
                        field_name: "_padding".into(),
                        value: ShaderParam::I32(0),
                    };

                    ShaderParam::Struct(vec![top, left, rotation, padding])
                }

                let layouts: Vec<ShaderParam> = textures_layouts
                    .iter()
                    .map(|layout| shader_texture_layout(layout, output_resolution))
                    .collect();

                Some(ShaderParam::List(layouts))
            }
        }
    }

    pub fn clear_color(transformation: &BuiltinTransformationSpec) -> Option<wgpu::Color> {
        match transformation {
            BuiltinTransformationSpec::TransformToResolution(TransformToResolution::Fit(
                background_color_rgba,
            )) => Some(rgba_to_wgpu_color(background_color_rgba)),
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
