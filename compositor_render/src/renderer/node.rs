use compositor_common::renderer_spec::RendererId;

use crate::transformations::shader::validation::ParametersValidationError;

// TODO: move here Node and RenderNode types

#[derive(Debug, thiserror::Error)]
pub enum CreateNodeError {
    #[error("Shader \"{0}\" does not exist. You have to register it first before using it in the scene definition.")]
    ShaderNotFound(RendererId),

    #[error("Error while validating the parameters for the shader node:\n{0}")]
    ShaderNodeParametersValidationError(#[from] ParametersValidationError),

    #[error("Instance of web renderer \"{0}\" does not exist. You have to register it first before using it in the scene definition.")]
    WebRendererNotFound(RendererId),

    #[error("Image \"{0}\" does not exist. You have to register it first before using it in the scene definition.")]
    ImageNotFound(RendererId),
}
