use compositor_common::scene::{InputId, OutputId};
use compositor_render::{
    error::{InitRendererEngineError, RegisterRendererError},
    registry::RegisterError,
    renderer::{scene::UpdateSceneError, WgpuError},
};

#[derive(Debug, thiserror::Error)]
pub enum RegisterInputError {
    #[error("Failed to register input stream. Stream \"{0}\" is already registered.")]
    AlreadyRegistered(InputId),
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterOutputError {
    #[error("Failed to register output stream. Stream \"{0}\" is already registered.")]
    AlreadyRegistered(OutputId),
}

#[derive(Debug, thiserror::Error)]
pub enum UnregisterInputError {
    #[error("Failed to unregister input stream. Stream \"{0}\" does not exist.")]
    NotFound(InputId),

    #[error(
        "Failed to unregister input stream. Stream \"{0}\" is still used in the current scene."
    )]
    StillInUse(InputId),
}

#[derive(Debug, thiserror::Error)]
pub enum UnregisterOutputError {
    #[error("Failed to unregister output stream. Stream \"{0}\" does not exist.")]
    NotFound(OutputId),

    #[error(
        "Failed to unregister output stream. Stream \"{0}\" is still used in the current scene."
    )]
    StillInUse(OutputId),
}

pub enum ErrorType {
    UserError,
    ServerError,
    EntityNotFound,
}

pub struct PipelineErrorInfo {
    pub error_code: &'static str,
    pub error_type: ErrorType,
}

impl PipelineErrorInfo {
    fn new(error_code: &'static str, error_type: ErrorType) -> Self {
        Self {
            error_code,
            error_type,
        }
    }
}

const INPUT_STREAM_ALREADY_REGISTERED: &str = "INPUT_STREAM_ALREADY_REGISTERED";

impl From<&RegisterInputError> for PipelineErrorInfo {
    fn from(err: &RegisterInputError) -> Self {
        match err {
            RegisterInputError::AlreadyRegistered(_) => {
                PipelineErrorInfo::new(INPUT_STREAM_ALREADY_REGISTERED, ErrorType::UserError)
            }
        }
    }
}

const OUTPUT_STREAM_ALREADY_REGISTERED: &str = "OUTPUT_STREAM_ALREADY_REGISTERED";

impl From<&RegisterOutputError> for PipelineErrorInfo {
    fn from(err: &RegisterOutputError) -> Self {
        match err {
            RegisterOutputError::AlreadyRegistered(_) => {
                PipelineErrorInfo::new(OUTPUT_STREAM_ALREADY_REGISTERED, ErrorType::UserError)
            }
        }
    }
}

const INPUT_STREAM_STILL_IN_USE: &str = "INPUT_STREAM_STILL_IN_USE";
const INPUT_STREAM_NOT_FOUND: &str = "INPUT_STREAM_NOT_FOUND";

impl From<&UnregisterInputError> for PipelineErrorInfo {
    fn from(err: &UnregisterInputError) -> Self {
        match err {
            UnregisterInputError::NotFound(_) => {
                PipelineErrorInfo::new(INPUT_STREAM_NOT_FOUND, ErrorType::EntityNotFound)
            }
            UnregisterInputError::StillInUse(_) => {
                PipelineErrorInfo::new(INPUT_STREAM_STILL_IN_USE, ErrorType::UserError)
            }
        }
    }
}

const OUTPUT_STREAM_STILL_IN_USE: &str = "OUTPUT_STREAM_STILL_IN_USE";
const OUTPUT_STREAM_NOT_FOUND: &str = "OUTPUT_STREAM_NOT_FOUND";

impl From<&UnregisterOutputError> for PipelineErrorInfo {
    fn from(err: &UnregisterOutputError) -> Self {
        match err {
            UnregisterOutputError::NotFound(_) => {
                PipelineErrorInfo::new(OUTPUT_STREAM_NOT_FOUND, ErrorType::EntityNotFound)
            }
            UnregisterOutputError::StillInUse(_) => {
                PipelineErrorInfo::new(OUTPUT_STREAM_STILL_IN_USE, ErrorType::UserError)
            }
        }
    }
}

const FAILED_TO_CREATE_NODE: &str = "FAILED_TO_CREATE_NODE";
const SCENE_SPEC_VALIDATION_ERROR: &str = "SCENE_SPEC_VALIDATION_ERROR";
const MISSING_NODE_WITH_ID: &str = "MISSING_NODE_WITH_ID";
const UNKNOWN_RESOLUTION_ON_OUTPUT_NODE: &str = "UNKNOWN_RESOLUTION_ON_OUTPUT_NODE";

impl From<&UpdateSceneError> for PipelineErrorInfo {
    fn from(err: &UpdateSceneError) -> Self {
        match err {
            UpdateSceneError::CreateNodeError(_, _) => {
                PipelineErrorInfo::new(FAILED_TO_CREATE_NODE, ErrorType::UserError)
            }
            UpdateSceneError::InvalidSpec(_) => {
                PipelineErrorInfo::new(SCENE_SPEC_VALIDATION_ERROR, ErrorType::UserError)
            }
            UpdateSceneError::NoNodeWithIdError(_) => {
                // ServerError because it should be validated is spec validation
                PipelineErrorInfo::new(MISSING_NODE_WITH_ID, ErrorType::ServerError)
            }
            UpdateSceneError::WgpuError(err) => err.into(),
            UpdateSceneError::UnknownResolutionOnOutput(_) => {
                PipelineErrorInfo::new(UNKNOWN_RESOLUTION_ON_OUTPUT_NODE, ErrorType::ServerError)
            }
        }
    }
}

const WGPU_INIT_ERROR: &str = "WGPU_INIT_ERROR";
const WEB_RENDERER_INIT_ERROR: &str = "WEB_RENDERER_INIT_ERROR";
const BUILTIN_INIT_ERROR: &str = "BUILTIN_INIT_ERROR";

impl From<&InitRendererEngineError> for PipelineErrorInfo {
    fn from(err: &InitRendererEngineError) -> Self {
        match err {
            InitRendererEngineError::FailedToInitWgpuCtx(_) => {
                PipelineErrorInfo::new(WGPU_INIT_ERROR, ErrorType::ServerError)
            }
            InitRendererEngineError::FailedToInitChromiumCtx(_) => {
                PipelineErrorInfo::new(WEB_RENDERER_INIT_ERROR, ErrorType::ServerError)
            }
            InitRendererEngineError::BuiltInTransformationsInitError(_) => {
                PipelineErrorInfo::new(BUILTIN_INIT_ERROR, ErrorType::ServerError)
            }
        }
    }
}

const ENTITY_ALREADY_REGISTERED: &str = "ENTITY_ALREADY_REGISTERED";
const INVALID_SHADER: &str = "INVALID_SHADER";
const REGISTER_IMAGE_ERROR: &str = "REGISTER_IMAGE_ERROR";

impl From<&RegisterRendererError> for PipelineErrorInfo {
    fn from(err: &RegisterRendererError) -> Self {
        match err {
            RegisterRendererError::RendererRegistry(err) => match err {
                RegisterError::KeyTaken { .. } => {
                    PipelineErrorInfo::new(ENTITY_ALREADY_REGISTERED, ErrorType::UserError)
                }
            },
            RegisterRendererError::Shader(_, _) => {
                PipelineErrorInfo::new(INVALID_SHADER, ErrorType::UserError)
            }
            RegisterRendererError::Image(_, _) => {
                PipelineErrorInfo::new(REGISTER_IMAGE_ERROR, ErrorType::UserError)
            }
        }
    }
}

const WGPU_VALIDATION_ERROR: &str = "WGPU_VALIDATION_ERROR";
const WGPU_OUT_OF_MEMORY_ERROR: &str = "WGPU_OUT_OF_MEMORY_ERROR";

impl From<&WgpuError> for PipelineErrorInfo {
    fn from(err: &WgpuError) -> Self {
        match err {
            WgpuError::Validation(_) => {
                PipelineErrorInfo::new(WGPU_VALIDATION_ERROR, ErrorType::UserError)
            }
            WgpuError::OutOfMemory(_) => {
                PipelineErrorInfo::new(WGPU_OUT_OF_MEMORY_ERROR, ErrorType::ServerError)
            }
        }
    }
}
