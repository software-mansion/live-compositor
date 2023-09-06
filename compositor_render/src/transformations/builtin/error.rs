use crate::transformations::shader::CreateShaderError;

#[derive(Debug, thiserror::Error)]
pub enum InitBuiltinError {
    #[error("Failed to initialize fixed_position_layout transformation. {0}")]
    FixedPositionLayout(#[source] CreateShaderError),

    #[error("Failed to initialize fit transform_to_resolution transformation. {0}")]
    FitToResolution(#[source] CreateShaderError),

    #[error("Failed to initialize fill transform_to_resolution transformation. {0}")]
    FillToResolution(#[source] CreateShaderError),

    #[error("Failed to initialize stretch transform_to_resolution transformation. {0}")]
    StretchToResolution(#[source] CreateShaderError),
}
