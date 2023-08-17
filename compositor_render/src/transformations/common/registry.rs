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
            CommonTransformation::ConvertResolution(ConvertResolutionParams::CropToFit) => {
                self.convert_resolution.crop_to_fit.clone()
            }
            CommonTransformation::ConvertResolution(ConvertResolutionParams::FillToFit(_)) => {
                self.convert_resolution.fill_to_fit.clone()
            }
        }
    }

    pub fn params(
        transformation: &CommonTransformation,
        output_resolution: Resolution,
    ) -> Option<ShaderParam> {
        match transformation {
            CommonTransformation::ConvertResolution(ConvertResolutionParams::Stretch) => None,
            CommonTransformation::ConvertResolution(ConvertResolutionParams::CropToFit) => {
                Some(ShaderParam::List(vec![
                    ShaderParam::U32(output_resolution.width as u32),
                    ShaderParam::U32(output_resolution.height as u32),
                ]))
            }
            CommonTransformation::ConvertResolution(ConvertResolutionParams::FillToFit(color)) => {
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
    crop_to_fit: Arc<Shader>,
    fill_to_fit: Arc<Shader>,
}

impl ConvertResolutionRegistry {
    pub(crate) fn new(wgpu_ctx: &Arc<WgpuCtx>) -> Self {
        Self {
            stretch: Arc::new(Shader::new(
                wgpu_ctx,
                include_str!("./convert_resolution/stretch.wgsl").into(),
            )),
            crop_to_fit: Arc::new(Shader::new(
                wgpu_ctx,
                include_str!("./convert_resolution/crop_to_fit.wgsl").into(),
            )),
            fill_to_fit: Arc::new(Shader::new(
                wgpu_ctx,
                include_str!("./convert_resolution/fill_to_fit.wgsl").into(),
            )),
        }
    }
}
