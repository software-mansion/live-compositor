pub mod event_loop;
pub mod frame_set;
pub mod registry;
pub mod renderer;

pub(crate) mod render_loop;
pub(crate) mod transformations;

mod sync_renderer;

pub use transformations::web_renderer::{
    browser::{EMBED_SOURCE_FRAMES_MESSAGE, SHMEM_FOLDER_PATH},
    WebRendererOptions,
};

pub type Renderer = sync_renderer::SyncRenderer;
