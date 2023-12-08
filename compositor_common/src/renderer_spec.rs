use std::{fmt::Display, sync::Arc};

use crate::scene::Resolution;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RendererId(pub Arc<str>);

impl Display for RendererId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FallbackStrategy {
    NeverFallback,
    FallbackIfAllInputsMissing,
    FallbackIfAnyInputMissing,
}

/// RendererSpec provides configuration necessary to construct Renderer. Renderers
/// are entities like shader, image or chromium_instance and can be used by nodes
/// to transform or generate frames.
#[derive(Debug)]
pub enum RendererSpec {
    Shader(ShaderSpec),
    WebRenderer(WebRendererSpec),
    Image(ImageSpec),
}

#[derive(Debug)]
pub struct ShaderSpec {
    pub shader_id: RendererId,
    pub source: String,
    pub fallback_strategy: FallbackStrategy,
}

#[derive(Debug)]
pub struct WebRendererSpec {
    pub instance_id: RendererId,
    pub url: String,
    pub resolution: Resolution,
    pub embedding_method: WebEmbeddingMethod,
    pub fallback_strategy: FallbackStrategy,
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

#[derive(Debug)]
pub struct ImageSpec {
    pub src: ImageSrc,
    pub image_id: RendererId,
    pub image_type: ImageType,
}

#[derive(Debug)]
pub enum ImageSrc {
    Url { url: String },
    LocalPath { path: String },
}

#[derive(Debug)]
pub enum ImageType {
    Png,
    Jpeg,
    Svg { resolution: Option<Resolution> },
    Gif,
}
