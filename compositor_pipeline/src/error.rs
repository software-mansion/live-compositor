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

pub struct PipelineError {
    pub error_code: &'static str,
    pub error_type: ErrorType,
    pub message: String,
}

impl PipelineError {
    fn new<T: std::error::Error>(error_code: &'static str, err: T, error_type: ErrorType) -> Self {
        Self {
            error_code,
            error_type,
            message: err.to_string(),
        }
    }
}

const INPUT_STREAM_ALREADY_REGISTERED: &str = "INPUT_STREAM_ALREADY_REGISTERED";

impl From<RegisterInputError> for PipelineError {
    fn from(err: RegisterInputError) -> Self {
        match err {
            RegisterInputError::AlreadyRegistered(_) => {
                PipelineError::new(INPUT_STREAM_ALREADY_REGISTERED, err, ErrorType::UserError)
            }
        }
    }
}

const OUTPUT_STREAM_ALREADY_REGISTERED: &str = "OUTPUT_STREAM_ALREADY_REGISTERED";

impl From<RegisterOutputError> for PipelineError {
    fn from(err: RegisterOutputError) -> Self {
        match err {
            RegisterOutputError::AlreadyRegistered(_) => {
                PipelineError::new(OUTPUT_STREAM_ALREADY_REGISTERED, err, ErrorType::UserError)
            }
        }
    }
}

const INPUT_STREAM_STILL_IN_USE: &str = "INPUT_STREAM_STILL_IN_USE";
const INPUT_STREAM_NOT_FOUND: &str = "INPUT_STREAM_NOT_FOUND";

impl From<UnregisterInputError> for PipelineError {
    fn from(err: UnregisterInputError) -> Self {
        match err {
            UnregisterInputError::NotFound(_) => {
                PipelineError::new(INPUT_STREAM_NOT_FOUND, err, ErrorType::EntityNotFound)
            }
            UnregisterInputError::StillInUse(_) => {
                PipelineError::new(INPUT_STREAM_STILL_IN_USE, err, ErrorType::UserError)
            }
        }
    }
}

const OUTPUT_STREAM_STILL_IN_USE: &str = "OUTPUT_STREAM_STILL_IN_USE";
const OUTPUT_STREAM_NOT_FOUND: &str = "OUTPUT_STREAM_NOT_FOUND";

impl From<UnregisterOutputError> for PipelineError {
    fn from(err: UnregisterOutputError) -> Self {
        match err {
            UnregisterOutputError::NotFound(_) => {
                PipelineError::new(OUTPUT_STREAM_NOT_FOUND, err, ErrorType::EntityNotFound)
            }
            UnregisterOutputError::StillInUse(_) => {
                PipelineError::new(OUTPUT_STREAM_STILL_IN_USE, err, ErrorType::UserError)
            }
        }
    }
}

const FAILED_TO_CREATE_NODE: &str = "FAILED_TO_CREATE_NODE";
const SCENE_SPEC_VALIDATION_ERROR: &str = "SCENE_SPEC_VALIDATION_ERROR";
const MISSING_NODE_WITH_ID: &str = "MISSING_NODE_WITH_ID";
const UNKNOWN_RESOLUTION_ON_OUTPUT_NODE: &str = "UNKNOWN_RESOLUTION_ON_OUTPUT_NODE";

impl From<UpdateSceneError> for PipelineError {
    fn from(err: UpdateSceneError) -> Self {
        match err {
            UpdateSceneError::CreateNodeError(_, _) => {
                PipelineError::new(FAILED_TO_CREATE_NODE, err, ErrorType::UserError)
            }
            UpdateSceneError::InvalidSpec(_) => {
                PipelineError::new(SCENE_SPEC_VALIDATION_ERROR, err, ErrorType::UserError)
            }
            UpdateSceneError::NoNodeWithIdError(_) => {
                // ServerError because it should be validated is spec validation
                PipelineError::new(MISSING_NODE_WITH_ID, err, ErrorType::ServerError)
            }
            UpdateSceneError::WgpuError(err) => err.into(),
            UpdateSceneError::UnknownResolutionOnOutput(_) => PipelineError::new(
                UNKNOWN_RESOLUTION_ON_OUTPUT_NODE,
                err,
                ErrorType::ServerError,
            ),
        }
    }
}

const WGPU_INIT_ERROR: &str = "WGPU_INIT_ERROR";
const WEB_RENDERER_INIT_ERROR: &str = "WEB_RENDERER_INIT_ERROR";
const BUILTIN_INIT_ERROR: &str = "BUILTIN_INIT_ERROR";

impl From<InitRendererEngineError> for PipelineError {
    fn from(err: InitRendererEngineError) -> Self {
        match err {
            InitRendererEngineError::FailedToInitWgpuCtx(_) => {
                PipelineError::new(WGPU_INIT_ERROR, err, ErrorType::ServerError)
            }
            InitRendererEngineError::FailedToInitChromiumCtx(_) => {
                PipelineError::new(WEB_RENDERER_INIT_ERROR, err, ErrorType::ServerError)
            }
            InitRendererEngineError::BuiltInTransformationsInitError(_) => {
                PipelineError::new(BUILTIN_INIT_ERROR, err, ErrorType::ServerError)
            }
        }
    }
}

const ENTITY_ALREADY_REGISTERED: &str = "ENTITY_ALREADY_REGISTERED";
const INVALID_SHADER: &str = "INVALID_SHADER";
const REGISTER_IMAGE_ERROR: &str = "REGISTER_IMAGE_ERROR";

impl From<RegisterRendererError> for PipelineError {
    fn from(err: RegisterRendererError) -> Self {
        match err {
            RegisterRendererError::RendererRegistry(err) => match err {
                RegisterError::KeyTaken { .. } => {
                    PipelineError::new(ENTITY_ALREADY_REGISTERED, err, ErrorType::UserError)
                }
            },
            RegisterRendererError::Shader(_) => {
                PipelineError::new(INVALID_SHADER, err, ErrorType::UserError)
            }
            RegisterRendererError::Image(_) => {
                PipelineError::new(REGISTER_IMAGE_ERROR, err, ErrorType::UserError)
            }
        }
    }
}

const WGPU_VALIDATION_ERROR: &str = "WGPU_VALIDATION_ERROR";
const WGPU_OUT_OF_MEMORY_ERROR: &str = "WGPU_OUT_OF_MEMORY_ERROR";

impl From<WgpuError> for PipelineError {
    fn from(err: WgpuError) -> Self {
        match err {
            WgpuError::Validation(_) => {
                PipelineError::new(WGPU_VALIDATION_ERROR, err, ErrorType::UserError)
            }
            WgpuError::OutOfMemory(_) => {
                PipelineError::new(WGPU_OUT_OF_MEMORY_ERROR, err, ErrorType::ServerError)
            }
        }
    }
}
