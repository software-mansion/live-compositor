use crate::{
    registry,
    renderer::{CreateWgpuCtxError, WgpuError},
    transformations::{
        builtin::transformations::InitBuiltinError,
        image_renderer::ImageError,
        shader::CreateShaderError,
        web_renderer::{chromium::WebRendererContextError, CreateWebRendererError},
    },
};

#[derive(Debug, thiserror::Error)]
pub enum InitRendererEngineError {
    #[error("Failed to initialize a wgpu context. {0}")]
    FailedToInitWgpuCtx(#[from] CreateWgpuCtxError),

    #[error("Failed to initialize chromium context. {0}")]
    FailedToInitChromiumCtx(#[from] WebRendererContextError),

    #[error(transparent)]
    BuiltInTransformationsInitError(#[from] InitBuiltinError),
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterRendererError {
    #[error(transparent)]
    RendererRegistry(#[from] registry::RegisterError),

    #[error(transparent)]
    Shader(#[from] CreateShaderError),

    #[error("Failed to create web renderer instance. {0}")]
    WebRendererInstance(#[from] CreateWebRendererError),

    #[error("Failed to prepare image.")]
    Image(#[from] ImageError),
}

#[derive(Debug, thiserror::Error)]
pub enum RenderSceneError {
    #[error(transparent)]
    WgpuError(#[from] WgpuError),
}
