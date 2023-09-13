use crate::transformations::shader_executor::CreateShaderError;

#[derive(Debug, thiserror::Error)]
pub enum InitBuiltinError {
    #[error(
        "Failed to initialize apply_transformation_matrix shader used in builtin transformations."
    )]
    ApplyTransformationMatrix(#[source] CreateShaderError),

    #[error("Failed to initialize mirror_image transformation.")]
    MirrorImage(#[source] CreateShaderError),

    #[error("Failed to initialize corners_rounding transformation.")]
    CornersRounding(#[source] CreateShaderError),
}
