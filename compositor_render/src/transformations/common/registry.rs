use std::sync::Arc;

use compositor_common::scene::{
    common_transformations::{CommonTransformation, ConvertResolutionParams},
    Resolution, ShaderParam,
};

use crate::{renderer::WgpuCtx, transformations::shader::Shader};

pub struct CommonTransformationsRegistry {
    convert_resolution: ConvertResolutionRegistry,
}

impl CommonTransformationsRegistry {
    pub fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Self {
        Self {
            convert_resolution: ConvertResolutionRegistry::new(wgpu_ctx),
        }
    }

    pub fn shader(&self, transformation: &CommonTransformation) -> Arc<Shader> {
        match transformation {
            CommonTransformation::ConvertResolution(ConvertResolutionParams::Stretch) => {
                self.convert_resolution.stretch.clone()
            }
            CommonTransformation::ConvertResolution(ConvertResolutionParams::CropScale) => {
                self.convert_resolution.crop_scale.clone()
            }
            CommonTransformation::ConvertResolution(ConvertResolutionParams::FillScale(_)) => {
                self.convert_resolution.fill_scale.clone()
            }
        }
    }

    pub fn params(
        transformation: &CommonTransformation,
        output_resolution: Resolution,
    ) -> Option<ShaderParam> {
        match transformation {
            CommonTransformation::ConvertResolution(ConvertResolutionParams::Stretch) => None,
            CommonTransformation::ConvertResolution(ConvertResolutionParams::CropScale) => {
                Some(ShaderParam::List(vec![
                    ShaderParam::U32(output_resolution.width as u32),
                    ShaderParam::U32(output_resolution.height as u32),
                ]))
            }
            CommonTransformation::ConvertResolution(ConvertResolutionParams::FillScale(color)) => {
                Some(ShaderParam::List(vec![
                    ShaderParam::U32(output_resolution.width as u32),
                    ShaderParam::U32(output_resolution.height as u32),
                    ShaderParam::U32(color.0 as u32),
                    ShaderParam::U32(color.1 as u32),
                    ShaderParam::U32(color.2 as u32),
                    ShaderParam::U32(color.3 as u32),
                ]))
            }
        }
    }
}

pub struct ConvertResolutionRegistry {
    stretch: Arc<Shader>,
    crop_scale: Arc<Shader>,
    fill_scale: Arc<Shader>,
}

impl ConvertResolutionRegistry {
    pub(crate) fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Self {
        Self {
            stretch: Arc::new(Shader::new(
                wgpu_ctx,
                include_str!("./convert_resolution/stretch.wgsl").into(),
            )),
            crop_scale: Arc::new(Shader::new(
                wgpu_ctx,
                include_str!("./convert_resolution/crop_scale.wgsl").into(),
            )),
            fill_scale: Arc::new(Shader::new(
                wgpu_ctx,
                include_str!("./convert_resolution/fill_scale.wgsl").into(),
            )),
        }
    }
}
