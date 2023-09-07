use crate::transformations::shader::CreateShaderError;

#[derive(Debug, thiserror::Error)]
pub enum InitBuiltinError {
    #[error("Failed to initialize fixed_position_layout transformation.")]
    FixedPositionLayout(#[source] CreateShaderError),

    #[error("Failed to initialize fit transform_to_resolution transformation.")]
    FitToResolution(#[source] CreateShaderError),

    #[error("Failed to initialize fill transform_to_resolution transformation.")]
    FillToResolution(#[source] CreateShaderError),

    #[error("Failed to initialize stretch transform_to_resolution transformation.")]
    StretchToResolution(#[source] CreateShaderError),

    #[error("Failed to initialize grid transformation. {0}")]
    Grid(#[source] CreateShaderError),
}
