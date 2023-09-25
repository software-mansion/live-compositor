use compositor_common::{
    error::UnsatisfiedConstraintsError,
    renderer_spec::RendererId,
    scene::{transition::TransitionValidationError, NodeId},
    SceneSpecValidationError,
};

use crate::transformations::web_renderer::CreateWebRendererError;
use crate::{
    gpu_shader::{error::ParametersValidationError, CreateShaderError},
    registry,
    renderer::{CreateWgpuCtxError, WgpuError},
    transformations::{
        builtin::error::InitBuiltinError, image_renderer::ImageError,
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

    #[error("Failed to register web renderer \"{1}\".")]
    Web(#[source] CreateWebRendererError, RendererId),
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

#[derive(Debug, thiserror::Error)]
pub enum UpdateSceneError {
    #[error("Failed to create node \"{1}\".")]
    CreateNodeError(#[source] CreateNodeError, NodeId),

    #[error(transparent)]
    InvalidSpec(#[from] SceneSpecValidationError),

    #[error("Unknown node \"{0}\" used in scene.")]
    NoNodeWithIdError(NodeId),

    #[error(transparent)]
    WgpuError(#[from] WgpuError),

    #[error("Unknown resolution on the output node. Nodes that are declared as outputs need to have constant resolution that is the same as resolution of the output stream.")]
    UnknownResolutionOnOutput(NodeId),

    #[error("Constraints for node \"{1}\" are not satisfied.")]
    ConstraintsValidationError(#[source] UnsatisfiedConstraintsError, NodeId),
}

#[derive(Debug, thiserror::Error)]
pub enum CreateNodeError {
    #[error("Shader \"{0}\" does not exist. You have to register it first before using it in the scene definition.")]
    ShaderNotFound(RendererId),

    #[error("Invalid parameter passed to \"{1}\" shader.")]
    ShaderNodeParametersValidationError(#[source] ParametersValidationError, RendererId),

    #[error("Instance of web renderer \"{0}\" does not exist. You have to register it first before using it in the scene definition.")]
    WebRendererNotFound(RendererId),

    #[error("Image \"{0}\" does not exist. You have to register it first before using it in the scene definition.")]
    ImageNotFound(RendererId),

    #[error(transparent)]
    TransitionValidation(#[from] TransitionValidationError),
}
