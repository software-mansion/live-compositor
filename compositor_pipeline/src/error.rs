use compositor_common::scene::{InputId, OutputId};
use compositor_render::renderer::{
    scene::SceneUpdateError, RendererInitError, RendererRegisterError,
};

const INPUT_STREAM_ALREADY_REGISTERED: &str = "INPUT_STREAM_ALREADY_REGISTERED";

const OUTPUT_STREAM_ALREADY_REGISTERED: &str = "OUTPUT_STREAM_ALREADY_REGISTERED";

const INPUT_STREAM_STILL_IN_USE: &str = "INPUT_STREAM_STILL_IN_USE";
const INPUT_STREAM_NOT_FOUND: &str = "INPUT_STREAM_NOT_FOUND";

const OUTPUT_STREAM_STILL_IN_USE: &str = "OUTPUT_STREAM_STILL_IN_USE";
const OUTPUT_STREAM_NOT_FOUND: &str = "OUTPUT_STREAM_NOT_FOUND";

#[derive(Debug, thiserror::Error)]
pub enum RegisterInputError {
    #[error("Failed to register input stream. Stream with id \"{0}\" is already registered.")]
    AlreadyRegistered(InputId),
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterOutputError {
    #[error("Failed to register output stream. Stream with id \"{0}\" is already registered.")]
    AlreadyRegistered(OutputId),
}

#[derive(Debug, thiserror::Error)]
pub enum UnregisterInputError {
    #[error("Failed to unregister input stream. Stream with id \"{0}\" does not exist.")]
    NotFound(InputId),

    #[error("Failed to unregister input stream. Stream with id \"{0}\" is still used in the current scene.")]
    StillInUse(InputId),
}

#[derive(Debug, thiserror::Error)]
pub enum UnregisterOutputError {
    #[error("Failed to unregister output stream. Stream with id \"{0}\" does not exist.")]
    NotFound(OutputId),

    #[error(
        "Failed to unregister output stream. Stream with id \"{0}\" is still used in the current scene."
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

impl From<RegisterInputError> for PipelineError {
    fn from(err: RegisterInputError) -> Self {
        match err {
            RegisterInputError::AlreadyRegistered(_) => {
                PipelineError::new(INPUT_STREAM_ALREADY_REGISTERED, err, ErrorType::UserError)
            }
        }
    }
}

impl From<RegisterOutputError> for PipelineError {
    fn from(err: RegisterOutputError) -> Self {
        match err {
            RegisterOutputError::AlreadyRegistered(_) => {
                PipelineError::new(OUTPUT_STREAM_ALREADY_REGISTERED, err, ErrorType::UserError)
            }
        }
    }
}

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

// placeholders for now

impl From<RendererInitError> for PipelineError {
    fn from(err: RendererInitError) -> Self {
        PipelineError {
            error_code: "UNKNOWN_INIT_ERROR",
            error_type: ErrorType::ServerError,
            message: err.to_string(),
        }
    }
}

impl From<SceneUpdateError> for PipelineError {
    fn from(err: SceneUpdateError) -> Self {
        PipelineError {
            error_code: "UNKNOWN_SCENE_UPDATE_ERROR",
            error_type: ErrorType::ServerError,
            message: format!("{:?}", err),
        }
    }
}

impl From<RendererRegisterError> for PipelineError {
    fn from(err: RendererRegisterError) -> Self {
        PipelineError {
            error_code: "UNKNOWN_RENDERER_REGISTER_ERROR",
            error_type: ErrorType::ServerError,
            message: err.to_string(),
        }
    }
}
