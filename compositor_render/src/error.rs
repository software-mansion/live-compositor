use compositor_common::renderer_spec::RendererId;

use crate::{
    registry,
    renderer::{CreateWgpuCtxError, WgpuError},
    transformations::{
        builtin::error::InitBuiltinError, image_renderer::ImageError, shader::CreateShaderError,
        web_renderer::chromium_context::WebRendererContextError,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum InitRendererEngineError {
    #[error("Failed to initialize a wgpu context.")]
    FailedToInitWgpuCtx(#[from] CreateWgpuCtxError),

    #[error("Failed to initialize chromium context.")]
    FailedToInitChromiumCtx(#[from] WebRendererContextError),

    #[error(transparent)]
    BuiltInTransformationsInitError(#[from] InitBuiltinError),
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterRendererError {
    #[error(transparent)]
    RendererRegistry(#[from] registry::RegisterError),

    #[error("Failed to register shader \"{1}\".")]
    Shader(#[source] CreateShaderError, RendererId),

    #[error("Failed to register image \"{1}\".")]
    Image(#[source] ImageError, RendererId),
}

#[derive(Debug, thiserror::Error)]
pub enum RenderSceneError {
    #[error(transparent)]
    WgpuError(#[from] WgpuError),
}
