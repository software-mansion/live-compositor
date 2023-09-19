use bytes::Bytes;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::renderer::{
    texture::{BGRATexture, NodeTexture},
    BGRAToRGBAConverter, RegisterCtx, RenderCtx,
};

use crate::transformations::web_renderer::frame_embedder::{FrameEmbedder, FrameEmbedderError};
use compositor_common::{
    renderer_spec::{FallbackStrategy, WebRendererSpec},
    scene::{constraints::NodeConstraints, NodeId, Resolution},
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use crate::renderer::texture::RGBATexture;


pub mod browser_client;
pub mod chromium_context;
mod chromium_sender;
mod chromium_sender_thread;
mod frame_embedder;
pub(crate) mod node;
mod shared_memory;

pub const EMBED_SOURCE_FRAMES_MESSAGE: &str = "EMBED_SOURCE_FRAMES";
pub const UNEMBED_SOURCE_FRAMES_MESSAGE: &str = "UNEMBED_SOURCE_FRAMES";
pub const GET_FRAME_POSITIONS_MESSAGE: &str = "GET_FRAME_POSITIONS";

pub(super) type FrameBytes = Arc<Mutex<Bytes>>;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct WebRendererOptions {
    pub init: bool,
    pub disable_gpu: bool,
}

impl Default for WebRendererOptions {
    fn default() -> Self {
        Self {
            init: true,
            disable_gpu: false,
        }
    }
}

pub struct WebRenderer {
    spec: WebRendererSpec,
    frame_embedder: Mutex<FrameEmbedder>,
    frame_bytes: FrameBytes,
    rendering_mode: WebRenderingMode,

    bgra_texture: BGRATexture,
    _bgra_bind_group_layout: wgpu::BindGroupLayout,
    bgra_bind_group: wgpu::BindGroup,
    bgra_to_rgba: BGRAToRGBAConverter,
}

impl WebRenderer {
    pub fn new(ctx: &RegisterCtx, spec: WebRendererSpec) -> Result<Self, CreateWebRendererError> {
        info!("Starting web renderer for {}", &spec.url);

        let bgra_texture = BGRATexture::new(&ctx.wgpu_ctx, spec.resolution);
        let bgra_bind_group_layout = BGRATexture::new_bind_group_layout(&ctx.wgpu_ctx.device);
        let bgra_bind_group = bgra_texture.new_bind_group(&ctx.wgpu_ctx, &bgra_bind_group_layout);
        let bgra_to_rgba = BGRAToRGBAConverter::new(&ctx.wgpu_ctx.device, &bgra_bind_group_layout);

        let frame_bytes = Arc::new(Mutex::new(Bytes::new()));
        let frame_embedder = Mutex::new(FrameEmbedder::new(ctx, frame_bytes.clone(), spec.url.clone(), spec.resolution)?);

        let rendering_mode = WebRenderingMode::from_spec(&spec);

        Ok(Self {
            spec,
            frame_embedder,
            frame_bytes,
            rendering_mode,
            bgra_texture,
            _bgra_bind_group_layout: bgra_bind_group_layout,
            bgra_bind_group,
            bgra_to_rgba,
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
        let mut frame_embedder = self.frame_embedder.lock().unwrap();
        if self.rendering_mode == WebRenderingMode::NativeEmbedding {
            frame_embedder.native_embed(node_id.clone(), sources, buffers)?;
        }

        if let Some(frame) = self.retrieve_frame() {
            let clear_target_texture = self.spec.use_native_embedding || sources.is_empty();
            let target = target.ensure_size(ctx.wgpu_ctx, self.spec.resolution);

            if self.rendering_mode == WebRenderingMode::FrameEmbeddingOnTop {
                frame_embedder.embed(sources, target);
            }

            self.bgra_texture.upload(ctx.wgpu_ctx, &frame);
            self.bgra_to_rgba.convert(
                ctx.wgpu_ctx,
                (&self.bgra_texture, &self.bgra_bind_group),
                target.rgba_texture(),
                clear_target_texture,
            );

            if self.rendering_mode == WebRenderingMode::FrameEmbedding {
                frame_embedder.embed(sources, target);
            }
        }

        Ok(())
    }

    fn retrieve_frame(&self) -> Option<Bytes> {
        let frame_data = self.frame_bytes.lock().unwrap();
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

    pub fn constrains(&self) -> &NodeConstraints {
        &self.spec.constraints
    }
}

#[derive(Debug, PartialEq)]
enum WebRenderingMode {
    /// Send frames to chromium directly and render it on canvas
    NativeEmbedding,

    /// Render website to texture and then place sources onto the rendered texture
    FrameEmbeddingOnTop,

    /// Render sources to texture and then render website on top.
    /// The website's background has to be transparent
    FrameEmbedding
}

impl WebRenderingMode {
    fn from_spec(spec: &WebRendererSpec) -> Self {
        if spec.use_native_embedding {
            return Self::NativeEmbedding;
        }

        if spec.embed_on_top {
            Self::FrameEmbeddingOnTop
        } else {
            Self::FrameEmbedding
        }
    }
}


#[derive(Debug, thiserror::Error)]
pub enum CreateWebRendererError {
    #[error(transparent)]
    FrameEmbedderError(#[from] FrameEmbedderError),
}

#[derive(Debug, thiserror::Error)]
pub enum RenderWebsiteError {
    #[error("Failed to embed sources")]
    EmbedSources(#[from] FrameEmbedderError),

    #[error("Download buffer does not exist")]
    ExpectDownloadBuffer,
}
