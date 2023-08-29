use std::sync::Arc;

use compositor_common::renderer_spec::RendererId;

use crate::{
    registry::{RegisterError, RegistryType, RendererRegistry},
    transformations::{
        builtin::transformations::BuiltinTransformations, image_renderer::Image, shader::Shader,
        web_renderer::WebRenderer,
    },
};

use super::{WgpuCtx, WgpuError};

pub(crate) struct TransformationsRegistry {
    pub(crate) shaders: RendererRegistry<Arc<Shader>>,
    pub(crate) web_renderers: RendererRegistry<Arc<WebRenderer>>,
    pub(crate) images: RendererRegistry<Image>,
    pub(crate) builtin: BuiltinTransformations,
}

impl TransformationsRegistry {
    pub(crate) fn new(wgpu_ctx: Arc<WgpuCtx>) -> Result<Self, WgpuError> {
        Ok(Self {
            shaders: RendererRegistry::new(RegistryType::Shader),
            web_renderers: RendererRegistry::new(RegistryType::WebRenderer),
            images: RendererRegistry::new(RegistryType::Image),
            builtin: BuiltinTransformations::new(&wgpu_ctx)?,
        })
    }

    pub(crate) fn register(&mut self, entry: RegistryEntry) -> Result<(), RegisterError> {
        match entry {
            RegistryEntry::Shader(shader_id, shader) => {
                Ok(self.shaders.register(shader_id, shader)?)
            }
            RegistryEntry::WebRenderer(instance_id, web_renderer) => {
                Ok(self.web_renderers.register(instance_id, web_renderer)?)
            }
            RegistryEntry::Image(image_id, image) => Ok(self.images.register(image_id, image)?),
        }
    }
}

pub(crate) enum RegistryEntry {
    Shader(RendererId, Arc<Shader>),
    WebRenderer(RendererId, Arc<WebRenderer>),
    Image(RendererId, Image),
}
