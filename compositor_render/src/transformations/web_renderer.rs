use crate::{RendererId, Resolution};

#[cfg(feature = "web_renderer")]
mod renderer;

#[cfg(not(feature = "web_renderer"))]
#[path = "web_renderer/dummy_renderer.rs"]
mod renderer;

pub use renderer::*;

#[cfg(feature = "web_renderer")]
pub mod chromium_context;
#[cfg(not(feature = "web_renderer"))]
#[path = "web_renderer/dummy_chromium_context.rs"]
pub mod chromium_context;

pub(crate) mod node;
#[cfg(feature = "web_renderer")]
mod web_renderer_thread;

#[cfg(feature = "web_renderer")]
mod render_info;
#[cfg(feature = "web_renderer")]
mod shader;
#[cfg(feature = "web_renderer")]
mod textures;

#[derive(Debug, Clone, Copy)]
pub struct WebRendererInitOptions {
    pub enable: bool,
    pub enable_gpu: bool,
}

#[derive(Debug, Clone)]
pub struct WebRendererSpec {
    pub instance_id: RendererId,
    pub url: String,
    pub resolution: Resolution,
    pub embedding_method: WebEmbeddingMethod,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WebEmbeddingMethod {
    /// Send frames to chromium directly and render it on canvas
    ChromiumEmbedding,

    /// Render sources on top of the rendered website
    NativeEmbeddingOverContent,

    /// Render sources below the website.
    /// The website's background has to be transparent
    NativeEmbeddingUnderContent,
}
