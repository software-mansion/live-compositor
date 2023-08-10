pub mod frame_set;
pub mod registry;
pub mod renderer;

pub(crate) mod render_loop;
pub(crate) mod transformations;

mod sync_renderer;

pub type Renderer = sync_renderer::SyncRenderer;
pub type EventLoop = transformations::web_renderer::chromium::EventLoop;
