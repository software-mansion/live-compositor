use crate::state::render_graph::NodeId;
use crate::state::{RegisterCtx, RenderCtx};
use crate::wgpu::common_pipeline::CreateShaderError;
use crate::wgpu::texture::NodeTexture;

use crate::{FallbackStrategy, RendererId, Resolution};
use bytes::Bytes;
use nalgebra_glm::Mat4;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[cfg(feature = "web_renderer")]
mod imports {
    pub(super) use crate::transformations::web_renderer::{
        browser_client::BrowserClient,
        chromium_sender::ChromiumSender,
        embedder::{EmbedError, EmbeddingHelper, RenderInfo},
        shader::WebRendererShader,
    };
    pub(super) use crate::wgpu::texture::{BGRATexture, Texture};
}

#[cfg(not(feature = "web_renderer"))]
mod imports {}

use imports::*;

use log::{error, info};

#[cfg(feature = "web_renderer")]
pub mod browser_client;
#[cfg(feature = "web_renderer")]
mod chromium_sender;
#[cfg(feature = "web_renderer")]
mod chromium_sender_thread;
#[cfg(feature = "web_renderer")]
mod embedder;
#[cfg(feature = "web_renderer")]
mod shader;
#[cfg(feature = "web_renderer")]
mod shared_memory;

pub mod chromium_context;
pub(crate) mod node;

pub const EMBED_SOURCE_FRAMES_MESSAGE: &str = "EMBED_SOURCE_FRAMES";
pub const UNEMBED_SOURCE_FRAMES_MESSAGE: &str = "UNEMBED_SOURCE_FRAMES";
pub const GET_FRAME_POSITIONS_MESSAGE: &str = "GET_FRAME_POSITIONS";

pub(super) type FrameData = Arc<Mutex<Bytes>>;
pub(super) type SourceTransforms = Arc<Mutex<Vec<Mat4>>>;

pub struct WebRendererInitOptions {
    pub init: bool,
    pub disable_gpu: bool,
}

impl Default for WebRendererInitOptions {
    fn default() -> Self {
        Self {
            init: true,
            disable_gpu: false,
        }
    }
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

#[cfg(feature = "web_renderer")]
#[derive(Debug)]
pub struct WebRenderer {
    spec: WebRendererSpec,
    frame_data: FrameData,
    source_transforms: SourceTransforms,
    embedding_helper: EmbeddingHelper,

    website_texture: BGRATexture,
    render_website_shader: WebRendererShader,
}

#[cfg(not(feature = "web_renderer"))]
#[derive(Debug)]
pub struct WebRenderer {
    spec: WebRendererSpec,
}

#[cfg(feature = "web_renderer")]
impl WebRenderer {
    pub fn new(ctx: &RegisterCtx, spec: WebRendererSpec) -> Result<Self, CreateWebRendererError> {
        if ctx.chromium.context.is_none() {
            return Err(CreateWebRendererError::WebRendererDisabled);
        }

        info!("Starting web renderer for {}", &spec.url);

        let frame_data = Arc::new(Mutex::new(Bytes::new()));
        let source_transforms = Arc::new(Mutex::new(Vec::new()));

        let client = BrowserClient::new(
            frame_data.clone(),
            source_transforms.clone(),
            spec.resolution,
        );
        let chromium_sender = ChromiumSender::new(ctx, spec.url.clone(), client);
        let embedding_helper = EmbeddingHelper::new(ctx, chromium_sender, spec.embedding_method);
        let render_website_shader = WebRendererShader::new(&ctx.wgpu_ctx)?;
        let website_texture = BGRATexture::new(&ctx.wgpu_ctx, spec.resolution);

        Ok(Self {
            spec,
            frame_data,
            source_transforms,
            embedding_helper,
            website_texture,
            render_website_shader,
        })
    }

    pub fn render(
        &self,
        ctx: &RenderCtx,
        node_id: &NodeId,
        sources: &[(&NodeId, &NodeTexture)],
        buffers: &[Arc<wgpu::Buffer>],
        target: &mut NodeTexture,
    ) -> Result<(), RenderWebsiteError> {
        self.embedding_helper
            .prepare_embedding(node_id, sources, buffers)
            .map_err(|err| RenderWebsiteError::EmbeddingFailed(self.spec.url.clone(), err))?;

        if let Some(frame) = self.retrieve_frame() {
            let target = target.ensure_size(ctx.wgpu_ctx, self.spec.resolution);
            self.website_texture.upload(ctx.wgpu_ctx, &frame);

            let render_textures = self.prepare_textures(sources);

            self.render_website_shader
                .render(ctx.wgpu_ctx, &render_textures, target);
        }

        Ok(())
    }

    fn prepare_textures<'a>(
        &'a self,
        sources: &'a [(&NodeId, &NodeTexture)],
    ) -> Vec<(Option<&Texture>, RenderInfo)> {
        let mut source_info = sources
            .iter()
            .zip(self.source_transforms.lock().unwrap().iter())
            .map(|((_node_id, node_texture), transform)| {
                (
                    node_texture.texture(),
                    RenderInfo::source_transform(transform),
                )
            })
            .collect();

        let website_info = (Some(self.website_texture.texture()), RenderInfo::website());

        let mut result = Vec::new();
        match self.spec.embedding_method {
            WebEmbeddingMethod::NativeEmbeddingOverContent => {
                result.push(website_info);
                result.append(&mut source_info);
            }
            WebEmbeddingMethod::NativeEmbeddingUnderContent => {
                result.append(&mut source_info);
                result.push(website_info);
            }
            WebEmbeddingMethod::ChromiumEmbedding => {
                result.push(website_info);
            }
        };

        result
    }

    fn retrieve_frame(&self) -> Option<Bytes> {
        let frame_data = self.frame_data.lock().unwrap();
        if frame_data.is_empty() {
            return None;
        }
        Some(frame_data.clone())
    }

    pub fn resolution(&self) -> Resolution {
        self.spec.resolution
    }

    pub fn shared_memory_root_path(renderer_id: &str) -> PathBuf {
        env::temp_dir()
            .join("video_compositor")
            .join(format!("instance_{}", renderer_id))
    }

    pub fn fallback_strategy(&self) -> FallbackStrategy {
        self.spec.fallback_strategy
    }
}

#[cfg(not(feature = "web_renderer"))]
impl WebRenderer {
    pub fn new(_ctx: &RegisterCtx, _spec: WebRendererSpec) -> Result<Self, CreateWebRendererError> {
        return Err(CreateWebRendererError::WebRenderingNotAvailable);
    }

    pub fn render(
        &self,
        _ctx: &RenderCtx,
        _node_id: &NodeId,
        _sources: &[(&NodeId, &NodeTexture)],
        _buffers: &[Arc<wgpu::Buffer>],
        _target: &mut NodeTexture,
    ) -> Result<(), RenderWebsiteError> {
        Ok(())
    }

    pub fn resolution(&self) -> Resolution {
        self.spec.resolution
    }

    pub fn fallback_strategy(&self) -> FallbackStrategy {
        self.spec.fallback_strategy
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateWebRendererError {
    #[error(transparent)]
    CreateShaderFailed(#[from] CreateShaderError),

    #[error("Web rendering can not be used because it was disabled in the init request")]
    WebRendererDisabled,

    #[error("Web rendering feature is not available")]
    WebRenderingNotAvailable,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderWebsiteError {
    #[cfg(feature = "web_renderer")]
    #[error("Failed to embed source on the website \"{0}\": {1}")]
    EmbeddingFailed(String, #[source] EmbedError),
}
