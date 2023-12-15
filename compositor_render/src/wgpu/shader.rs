use super::{validation::ShaderValidationError, WgpuError};

pub const VERTEX_ENTRYPOINT_NAME: &str = "vs_main";
pub const FRAGMENT_ENTRYPOINT_NAME: &str = "fs_main";

pub const USER_DEFINED_BUFFER_GROUP: u32 = 1;
pub const USER_DEFINED_BUFFER_BINDING: u32 = 0;

#[derive(Debug, thiserror::Error)]
pub enum CreateShaderError {
    #[error(transparent)]
    Wgpu(#[from] WgpuError),

    #[error(transparent)]
    Validation(#[from] ShaderValidationError),

    #[error("Shader parse error: {0}")]
    ParseError(naga::front::wgsl::ParseError),
}
