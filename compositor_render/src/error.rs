use compositor_common::{
    error::UnsatisfiedConstraintsError,
    renderer_spec::RendererId,
    scene::{transition::TransitionValidationError, OutputId},
    SceneSpecValidationError,
};

use crate::{
    registry,
    scene::BuildSceneError,
    transformations::{
        builtin::error::InitBuiltinError, image_renderer::ImageError,
        web_renderer::chromium_context::WebRendererContextError,
    },
    wgpu::{shader::CreateShaderError, validation::ParametersValidationError, CreateWgpuCtxError},
};
use crate::{
    renderer::render_graph::NodeId, transformations::web_renderer::CreateWebRendererError,
};

pub use crate::registry::RegisterError;
pub use crate::wgpu::WgpuError;

#[derive(Debug, thiserror::Error)]
pub enum InitRendererEngineError {
    #[error("Failed to initialize a wgpu context.")]
    FailedToInitWgpuCtx(#[from] CreateWgpuCtxError),

    #[error("Failed to initialize chromium context.")]
    FailedToInitChromiumCtx(#[from] WebRendererContextError),

    #[error(transparent)]
    BuiltInTransformationsInitError(#[from] InitBuiltinError),

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
    CreateNodeError(#[source] CreateNodeError, usize),

    #[error(transparent)]
    InvalidSpec(#[from] SceneSpecValidationError),

    #[error("Unknown node \"{0}\" used in scene.")]
    NoNodeWithIdError(NodeId),

    #[error(transparent)]
    WgpuError(#[from] WgpuError),

    #[error(
        "Output \"{0}\" does not exist, register it first before using it in the scene definition"
    )]
    OutputNotRegistered(OutputId),

    #[error("Constraints for node \"{1}\" are not satisfied.")]
    ConstraintsValidationError(#[source] UnsatisfiedConstraintsError, NodeId),

    #[error(transparent)]
    BuildSceneError(#[from] BuildSceneError),
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
