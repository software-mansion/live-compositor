use compositor_common::{renderer_spec::RendererId, scene::NodeId};

use crate::{
    registry,
    renderer::{CreateWgpuCtxError, WgpuError},
    transformations::{
        builtin::error::InitBuiltinError, image_renderer::ImageError,
        shader_executor::CreateShaderError,
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
pub enum UnregisterRendererError {
    #[error(transparent)]
    RendererRegistry(#[from] registry::UnregisterError),

    #[error(
        "Failed to unregister \"{0}\" image. It is still used in scene definition by \"{1}\" node."
    )]
    ImageStillInUse(RendererId, NodeId),

    #[error("Failed to unregister \"{0}\" shader. It is still used in scene definition by \"{1}\" node.")]
    ShaderStillInUse(RendererId, NodeId),

    #[error(
        "Failed to unregister \"{0}\" web renderer instance. It is still used in scene definition by \"{1}\" node."
    )]
    WebRendererInstanceStillInUse(RendererId, NodeId),
}

#[derive(Debug, thiserror::Error)]
pub enum RenderSceneError {
    #[error(transparent)]
    WgpuError(#[from] WgpuError),
}
