pub mod error;
pub mod renderer;
pub mod scene;

pub(crate) mod registry;
pub(crate) mod transformations;
pub(crate) mod utils;
pub(crate) mod wgpu;

mod event_loop;
mod sync_renderer;
mod types;

pub use event_loop::EventLoop;
pub use types::*;

pub use transformations::web_renderer::{
    WebRendererOptions, EMBED_SOURCE_FRAMES_MESSAGE, GET_FRAME_POSITIONS_MESSAGE,
    UNEMBED_SOURCE_FRAMES_MESSAGE,
};

pub type Renderer = sync_renderer::SyncRenderer;
pub use registry::RegistryType;
pub use sync_renderer::RendererSpec;

pub use transformations::image_renderer::{ImageSpec, ImageSrc, ImageType};
pub use transformations::shader::ShaderSpec;
pub use transformations::web_renderer::{WebEmbeddingMethod, WebRendererSpec};
