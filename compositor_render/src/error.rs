use compositor_common::{renderer_spec::RendererId, scene::OutputId};

use crate::transformations::web_renderer::CreateWebRendererError;
use crate::{
    registry,
    scene::SceneError,
    transformations::{
        image_renderer::ImageError, web_renderer::chromium_context::WebRendererContextError,
    },
    wgpu::{shader::CreateShaderError, CreateWgpuCtxError},
};

pub use crate::registry::RegisterError;
pub use crate::wgpu::WgpuError;

#[derive(Debug, thiserror::Error)]
pub enum InitRendererEngineError {
    #[error("Failed to initialize a wgpu context.")]
    FailedToInitWgpuCtx(#[from] CreateWgpuCtxError),

    #[error("Failed to initialize chromium context.")]
    FailedToInitChromiumCtx(#[from] WebRendererContextError),

    #[error("Failed to initialize apply_layout transformation.")]
    LayoutTransformationsInitError(#[source] CreateShaderError),
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterRendererError {
    #[error(transparent)]
    RendererRegistry(#[from] registry::RegisterError),

    #[error("Failed to register shader \"{1}\".")]
    Shader(#[source] CreateShaderError, RendererId),

    #[error("Failed to register image \"{1}\".")]
    Image(#[source] ImageError, RendererId),

    #[error("Failed to register web renderer \"{1}\".")]
    Web(#[source] CreateWebRendererError, RendererId),
}

#[derive(Debug, thiserror::Error)]
pub enum UnregisterRendererError {
    #[error(transparent)]
    RendererRegistry(#[from] registry::UnregisterError),
}

#[derive(Debug, thiserror::Error)]
pub enum RenderSceneError {
    #[error(transparent)]
    WgpuError(#[from] WgpuError),
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateSceneError {
    #[error(transparent)]
    WgpuError(#[from] WgpuError),

    #[error(
        "Output \"{0}\" does not exist, register it first before using it in the scene definition."
    )]
    OutputNotRegistered(OutputId),

    #[error(transparent)]
    SceneError(#[from] SceneError),
}
