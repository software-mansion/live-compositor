use std::sync::Arc;

use crate::{
    registry::{RegistryType, RendererRegistry},
    transformations::{
        builtin::{error::InitBuiltinError, transformations::BuiltinTransformations},
        image_renderer::Image,
        shader::Shader,
        web_renderer::WebRenderer,
    },
};

use super::WgpuCtx;

pub struct Renderers {
    pub shaders: RendererRegistry<Arc<Shader>>,
    pub web_renderers: RendererRegistry<Arc<WebRenderer>>,
    pub images: RendererRegistry<Image>,
    pub builtin: BuiltinTransformations,
}

impl Renderers {
    pub fn new(wgpu_ctx: Arc<WgpuCtx>) -> Result<Self, InitBuiltinError> {
        Ok(Self {
            shaders: RendererRegistry::new(RegistryType::Shader),
            web_renderers: RendererRegistry::new(RegistryType::WebRenderer),
            images: RendererRegistry::new(RegistryType::Image),
            builtin: BuiltinTransformations::new(&wgpu_ctx)?,
        })
    }
}
