use std::sync::Arc;

use crate::{
    error::InitRendererEngineError,
    registry::{RegistryType, RendererRegistry},
    transformations::{
        builtin::transformations::BuiltinTransformations, image_renderer::Image,
        layout::LayoutRenderer, shader::Shader, web_renderer::WebRenderer,
    },
};

use super::WgpuCtx;

pub(crate) struct Renderers {
    pub(crate) shaders: RendererRegistry<Arc<Shader>>,
    pub(crate) web_renderers: RendererRegistry<Arc<WebRenderer>>,
    pub(crate) images: RendererRegistry<Image>,
    pub(crate) builtin: BuiltinTransformations,
    pub(crate) layout: LayoutRenderer,
}

impl Renderers {
    pub fn new(wgpu_ctx: Arc<WgpuCtx>) -> Result<Self, InitRendererEngineError> {
        Ok(Self {
            shaders: RendererRegistry::new(RegistryType::Shader),
            web_renderers: RendererRegistry::new(RegistryType::WebRenderer),
            images: RendererRegistry::new(RegistryType::Image),
            builtin: BuiltinTransformations::new(&wgpu_ctx)?,
            layout: LayoutRenderer::new(&wgpu_ctx)
                .map_err(InitRendererEngineError::LayoutTransformationsInitError)?,
        })
    }
}
