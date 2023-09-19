use bytes::Bytes;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::renderer::{
    texture::{BGRATexture, NodeTexture},
    BGRAToRGBAConverter, RegisterCtx, RenderCtx,
};

use crate::transformations::web_renderer::browser::BrowserClient;
use crate::transformations::web_renderer::frame_embedder::{FrameEmbedder, FrameEmbedderError};
use compositor_common::{
    renderer_spec::{FallbackStrategy, WebRendererSpec},
    scene::{constraints::NodeConstraints, NodeId, Resolution},
};
use log::{error, info};
use serde::{Deserialize, Serialize};

use self::browser::{BrowserController, EmbedFrameError};

pub mod browser;
pub mod chromium_context;
mod chromium_sender;
mod chromium_sender_thread;
mod frame_embedder;
pub(crate) mod node;
mod shared_memory;

pub const EMBED_SOURCE_FRAMES_MESSAGE: &str = "EMBED_SOURCE_FRAMES";
pub const UNEMBED_SOURCE_FRAMES_MESSAGE: &str = "UNEMBED_SOURCE_FRAMES";
pub const GET_FRAME_POSITIONS_MESSAGE: &str = "GET_FRAME_POSITIONS";

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
    controller: Mutex<BrowserController>,
    frame_embedder: Mutex<FrameEmbedder>,

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

        let frame_embedder = FrameEmbedder::new(&ctx.wgpu_ctx)?;
        let controller = Mutex::new(BrowserController::new(
            ctx,
            spec.url.clone(),
            spec.resolution,
            frame_embedder.frame_transforms(),
        ));

        Ok(Self {
            spec,
            controller,
            frame_embedder: Mutex::new(frame_embedder),
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
        let mut controller = self.controller.lock().unwrap();
        let mut frame_embedder = self.frame_embedder.lock().unwrap();
        if self.spec.use_native_embedding {
            controller.send_sources(ctx, node_id.clone(), sources, buffers)?;
        } else {
            controller.request_frame_positions(sources);
        }

        if let Some(frame) = controller.retrieve_frame() {
            let target = target.ensure_size(ctx.wgpu_ctx, self.spec.resolution);

            if !self.spec.use_native_embedding {
                frame_embedder.embed(sources, target);
            }
            self.bgra_texture.upload(ctx.wgpu_ctx, &frame);
            self.bgra_to_rgba.convert(
                ctx.wgpu_ctx,
                (&self.bgra_texture, &self.bgra_bind_group),
                target.rgba_texture(),
            );
        }

        Ok(())
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

#[derive(Debug, thiserror::Error)]
pub enum CreateWebRendererError {
    #[error(transparent)]
    FrameEmbedderError(#[from] FrameEmbedderError),
}

#[derive(Debug, thiserror::Error)]
pub enum RenderWebsiteError {
    #[error("Failed to embed sources")]
    EmbedSources(#[from] EmbedFrameError),

    #[error("Download buffer does not exist")]
    ExpectDownloadBuffer,
}
