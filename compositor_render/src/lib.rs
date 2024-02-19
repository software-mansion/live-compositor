pub mod error;
pub mod scene;

pub(crate) mod registry;
pub(crate) mod transformations;
pub(crate) mod utils;
pub(crate) mod wgpu;

mod event_loop;
mod state;
mod types;

pub use event_loop::EventLoop;
pub use types::*;

pub use registry::RegistryType;
pub use state::Renderer;
pub use state::RendererOptions;
pub use state::RendererSpec;

pub use state::AudioMixer;
pub use wgpu::use_global_wgpu_ctx;

pub mod image {
    pub use crate::transformations::image_renderer::{ImageSource, ImageSpec, ImageType};
}

pub mod shader {
    pub use crate::transformations::shader::ShaderSpec;
}

pub mod web_renderer {
    pub use crate::transformations::web_renderer::{
        WebEmbeddingMethod, WebRendererInitOptions, WebRendererSpec,
    };

    #[cfg(feature = "web_renderer")]
    pub use crate::transformations::web_renderer::{
        DROP_SHARED_MEMORY, EMBED_FRAMES_MESSAGE, GET_FRAME_POSITIONS_MESSAGE,
    };
}
