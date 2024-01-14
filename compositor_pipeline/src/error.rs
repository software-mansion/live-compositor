use compositor_render::{
    error::{
        InitRendererEngineError, RegisterError, RegisterRendererError, UnregisterRendererError,
        UpdateSceneError, WgpuError,
    },
    InputId, OutputId,
};

use crate::pipeline::{decoder::DecoderOptions, input::rtp::ChunkIter};

#[derive(Debug, thiserror::Error)]
pub enum RegisterInputError {
    #[error("Failed to register input stream. Stream \"{0}\" is already registered.")]
    AlreadyRegistered(InputId),

    #[error("Decoder error while registering input stream for stream \"{0}\".")]
    DecoderError(InputId, #[source] DecoderInitError),

    #[error("Input initialization error while registering input for stream \"{0}\".")]
    InputError(InputId, #[source] InputInitError),
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterOutputError {
    #[error("Failed to register output stream. Stream \"{0}\" is already registered.")]
    AlreadyRegistered(OutputId),

    #[error("Encoder error while registering output stream for stream \"{0}\".")]
    EncoderError(OutputId, #[source] OutputInitError),

    #[error("Failed to register output stream \"{0}\". Resolution in each dimension has to be divisible by 2.")]
    UnsupportedResolution(OutputId),
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

pub struct CustomError(pub Box<dyn std::error::Error + Send + Sync + 'static>);

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::fmt::Debug for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl std::error::Error for CustomError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OutputInitError {
    #[error("Could not find an ffmpeg codec")]
    NoCodec,

    #[error(transparent)]
    FfmpegError(#[from] ffmpeg_next::Error),

    #[error(transparent)]
    OutputError(#[from] CustomError),
}

#[derive(Debug, thiserror::Error)]
pub enum DecoderInitError {
    #[error(transparent)]
    FfmpegError(#[from] ffmpeg_next::Error),
    #[error(transparent)]
    OpusError(#[from] opus::Error),
    #[error("Decoder options doesn't match input chunk iter type. Chunk iter: {0:#?}, decoder options: {1:?}")]
    InvalidDecoderOptions(ChunkIter, DecoderOptions),
}

#[derive(Debug, thiserror::Error)]
pub enum InputInitError {
    #[error(transparent)]
    Rtp(#[from] crate::pipeline::input::rtp::RtpReceiverError),
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
const DECODER_ERROR: &str = "INPUT_STREAM_DECODER_ERROR";
const INPUT_ERROR: &str = "INPUT_STREAM_INPUT_ERROR";

impl From<&RegisterInputError> for PipelineErrorInfo {
    fn from(err: &RegisterInputError) -> Self {
        match err {
            RegisterInputError::AlreadyRegistered(_) => {
                PipelineErrorInfo::new(INPUT_STREAM_ALREADY_REGISTERED, ErrorType::UserError)
            }

            RegisterInputError::DecoderError(_, _) => {
                PipelineErrorInfo::new(DECODER_ERROR, ErrorType::ServerError)
            }

            RegisterInputError::InputError(_, _) => {
                PipelineErrorInfo::new(INPUT_ERROR, ErrorType::ServerError)
            }
        }
    }
}

const OUTPUT_STREAM_ALREADY_REGISTERED: &str = "OUTPUT_STREAM_ALREADY_REGISTERED";
const ENCODER_ERROR: &str = "ENCODER_ERROR";
const UNSUPPORTED_RESOLUTION: &str = "UNSUPPORTED_RESOLUTION";

impl From<&RegisterOutputError> for PipelineErrorInfo {
    fn from(err: &RegisterOutputError) -> Self {
        match err {
            RegisterOutputError::AlreadyRegistered(_) => {
                PipelineErrorInfo::new(OUTPUT_STREAM_ALREADY_REGISTERED, ErrorType::UserError)
            }

            RegisterOutputError::EncoderError(_, _) => {
                PipelineErrorInfo::new(ENCODER_ERROR, ErrorType::ServerError)
            }
            RegisterOutputError::UnsupportedResolution(_) => {
                PipelineErrorInfo::new(UNSUPPORTED_RESOLUTION, ErrorType::UserError)
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

const BUILD_SCENE_ERROR: &str = "BUILD_SCENE_ERROR";

impl From<&UpdateSceneError> for PipelineErrorInfo {
    fn from(err: &UpdateSceneError) -> Self {
        match err {
            UpdateSceneError::WgpuError(err) => err.into(),
            UpdateSceneError::OutputNotRegistered(_) => {
                PipelineErrorInfo::new(OUTPUT_STREAM_NOT_FOUND, ErrorType::UserError)
            }
            UpdateSceneError::SceneError(_) => PipelineErrorInfo {
                error_code: BUILD_SCENE_ERROR,
                error_type: ErrorType::UserError,
            },
        }
    }
}

const WGPU_INIT_ERROR: &str = "WGPU_INIT_ERROR";
const WEB_RENDERER_INIT_ERROR: &str = "WEB_RENDERER_INIT_ERROR";
const LAYOUT_INIT_ERROR: &str = "LAYOUT_INIT_ERROR";

impl From<&InitRendererEngineError> for PipelineErrorInfo {
    fn from(err: &InitRendererEngineError) -> Self {
        match err {
            InitRendererEngineError::FailedToInitWgpuCtx(_) => {
                PipelineErrorInfo::new(WGPU_INIT_ERROR, ErrorType::ServerError)
            }
            InitRendererEngineError::FailedToInitChromiumCtx(_) => {
                PipelineErrorInfo::new(WEB_RENDERER_INIT_ERROR, ErrorType::ServerError)
            }
            InitRendererEngineError::LayoutTransformationsInitError(_) => {
                PipelineErrorInfo::new(LAYOUT_INIT_ERROR, ErrorType::ServerError)
            }
        }
    }
}

const ENTITY_ALREADY_REGISTERED: &str = "ENTITY_ALREADY_REGISTERED";
const INVALID_SHADER: &str = "INVALID_SHADER";
const REGISTER_IMAGE_ERROR: &str = "REGISTER_IMAGE_ERROR";
const REGISTER_WEB_RENDERER_ERROR: &str = "REGISTER_WEB_RENDERER_ERROR";

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
            RegisterRendererError::Web(_, _) => {
                PipelineErrorInfo::new(REGISTER_WEB_RENDERER_ERROR, ErrorType::ServerError)
            }
        }
    }
}

const ENTITY_NOT_FOUND: &str = "ENTITY_NOT_FOUND";

impl From<&UnregisterRendererError> for PipelineErrorInfo {
    fn from(err: &UnregisterRendererError) -> Self {
        match err {
            UnregisterRendererError::RendererRegistry(_) => {
                PipelineErrorInfo::new(ENTITY_NOT_FOUND, ErrorType::EntityNotFound)
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
