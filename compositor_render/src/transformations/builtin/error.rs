use crate::transformations::shader::CreateShaderError;

#[derive(Debug, thiserror::Error)]
pub enum InitBuiltinError {
    #[error("Failed to initialize `apply_transformation_shader` used in builtin transformations.")]
    ApplyTransformationMatrix(#[source] CreateShaderError),

    #[error("Failed to initialize mirror_image transformation.")]
    MirrorImage(#[source] CreateShaderError),
}
