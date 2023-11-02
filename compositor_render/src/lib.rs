pub mod error;
pub mod renderer;

pub(crate) mod registry;
pub(crate) mod transformations;
pub(crate) mod utils;
pub(crate) mod wgpu;

mod event_loop;
mod frame_set;
mod sync_renderer;
mod validation;

pub use event_loop::EventLoop;
pub use frame_set::FrameSet;

pub use transformations::web_renderer::{
    WebRendererOptions, EMBED_SOURCES_MESSAGE, GET_FRAME_POSITIONS_MESSAGE, UNEMBED_SOURCE_MESSAGE,
};

pub type Renderer = sync_renderer::SyncRenderer;
pub use registry::RegistryType;
