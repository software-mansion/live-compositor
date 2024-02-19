use std::{env, path::PathBuf, sync::Arc, thread::JoinHandle};

use bytes::Bytes;
use crossbeam_channel::{SendError, Sender};
use log::{error, info};

use crate::{
    scene::ComponentId,
    state::{RegisterCtx, RenderCtx},
    transformations::{
        layout::{vertices_transformation_matrix, Position},
        web_renderer::web_renderer_thread::WebRendererThread,
    },
    wgpu::{
        common_pipeline::CreateShaderError,
        texture::{BGRATexture, NodeTexture, Texture},
        WgpuCtx,
    },
    Resolution,
};

use super::{
    node::EmbeddingData,
    render_info::RenderInfo,
    shader::WebRendererShader,
    web_renderer_thread::communication::{
        new_response_channel, ResponseReceiverError, UpdateSharedMemoryPayload,
        WebRendererThreadRequest,
    },
    WebEmbeddingMethod, WebRendererSpec,
};

pub use super::web_renderer_thread::communication::{
    DROP_SHARED_MEMORY, EMBED_FRAMES_MESSAGE, GET_FRAME_POSITIONS_MESSAGE,
};

#[derive(Debug)]
pub struct WebRenderer {
    spec: WebRendererSpec,

    website_texture: BGRATexture,
    render_website_shader: WebRendererShader,

    request_sender: Sender<WebRendererThreadRequest>,
    thread_join_handle: Option<JoinHandle<()>>,
}

impl Drop for WebRenderer {
    fn drop(&mut self) {
        if let Err(err) = self.request_sender.send(WebRendererThreadRequest::Quit) {
            error!("Failed to close WebRendererThread: {err}");
        }

        let Some(join_handle) = self.thread_join_handle.take() else {
            error!("WebRendererThread join handle not found");
            return;
        };

        join_handle.join().unwrap();
    }
}

impl WebRenderer {
    pub fn new(ctx: &RegisterCtx, spec: WebRendererSpec) -> Result<Self, CreateWebRendererError> {
        if ctx.chromium.context.is_none() {
            return Err(CreateWebRendererError::WebRendererDisabled);
        }

        info!("Starting web renderer for {}", &spec.url);

        let render_website_shader: WebRendererShader = WebRendererShader::new(&ctx.wgpu_ctx)?;
        let website_texture = BGRATexture::new(&ctx.wgpu_ctx, spec.resolution);

        let (request_sender, request_receiver) = crossbeam_channel::unbounded();
        let join_handle = WebRendererThread::new(ctx, spec.clone(), request_receiver).spawn();

        Ok(Self {
            spec,
            website_texture,
            render_website_shader,
            request_sender,
            thread_join_handle: Some(join_handle),
        })
    }

    pub fn render(
        &self,
        ctx: &RenderCtx,
        sources: &[&NodeTexture],
        embedding_data: &EmbeddingData,
        target: &mut NodeTexture,
    ) -> Result<(), RenderWebsiteError> {
        let frame_positions = match self.spec.embedding_method {
            WebEmbeddingMethod::NativeEmbeddingOverContent
            | WebEmbeddingMethod::NativeEmbeddingUnderContent => {
                self.frame_positions(embedding_data.children_ids.clone())?
            }
            WebEmbeddingMethod::ChromiumEmbedding => Vec::new(),
        };

        if self.spec.embedding_method == WebEmbeddingMethod::ChromiumEmbedding {
            self.embed_sources_with_chromium(ctx.wgpu_ctx, sources, embedding_data)?;
        }

        if let Some(frame) = self.frame_data()? {
            let target = target.ensure_size(ctx.wgpu_ctx, self.spec.resolution);
            self.website_texture.upload(ctx.wgpu_ctx, &frame);

            let render_textures = self.prepare_textures(sources, &frame_positions);
            self.render_website_shader
                .render(ctx.wgpu_ctx, &render_textures, target);
        }

        Ok(())
    }

    /// Send sources to chromium and render them on canvases via JS API
    pub fn embed_sources_with_chromium(
        &self,
        wgpu_ctx: &Arc<WgpuCtx>,
        sources: &[&NodeTexture],
        embedding_data: &EmbeddingData,
    ) -> Result<(), RenderWebsiteError> {
        self.ensure_shared_memory(sources)?;
        Self::copy_sources_to_buffers(wgpu_ctx, sources, &embedding_data.buffers);
        self.copy_buffers_to_shared_memory(wgpu_ctx, sources, &embedding_data.buffers)?;

        let resolutions = sources.iter().map(|texture| texture.resolution()).collect();
        self.request_sender
            .send(WebRendererThreadRequest::EmbedSources {
                resolutions,
                children_ids: embedding_data.children_ids.clone(),
            })?;

        Ok(())
    }

    fn copy_sources_to_buffers(
        wgpu_ctx: &Arc<WgpuCtx>,
        sources: &[&NodeTexture],
        buffers: &[Arc<wgpu::Buffer>],
    ) {
        let mut encoder = wgpu_ctx.device.create_command_encoder(&Default::default());
        for (texture, buffer) in sources.iter().zip(buffers) {
            let Some(texture_state) = texture.state() else {
                continue;
            };
            texture_state
                .rgba_texture()
                .copy_to_buffer(&mut encoder, buffer);
        }

        wgpu_ctx.queue.submit(Some(encoder.finish()));
    }

    fn copy_buffers_to_shared_memory(
        &self,
        wgpu_ctx: &Arc<WgpuCtx>,
        sources: &[&NodeTexture],
        buffers: &[Arc<wgpu::Buffer>],
    ) -> Result<(), RenderWebsiteError> {
        let mut pending_buffer_copies = Vec::new();
        for (source_idx, (texture, buffer)) in sources.iter().zip(buffers).enumerate() {
            let Some(texture_state) = texture.state() else {
                continue;
            };

            let size = texture_state.rgba_texture().size();
            let pending_copy = self.init_buffer_to_shmem_copy(source_idx, size, buffer.clone());
            pending_buffer_copies.push(pending_copy);
        }

        wgpu_ctx.device.poll(wgpu::Maintain::Wait);

        for pending in pending_buffer_copies {
            pending()?;
        }
        Ok(())
    }

    fn init_buffer_to_shmem_copy(
        &self,
        source_idx: usize,
        size: wgpu::Extent3d,
        buffer: Arc<wgpu::Buffer>,
    ) -> impl FnOnce() -> Result<(), RenderWebsiteError> + '_ {
        let (s, r) = crossbeam_channel::bounded(1);
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                if let Err(err) = s.send(result) {
                    error!("channel send error: {err}")
                }
            });

        move || {
            r.recv().unwrap()?;

            self.update_shared_memory(source_idx, buffer.clone(), size)?;
            buffer.unmap();

            Ok(())
        }
    }

    pub fn frame_data(&self) -> Result<Option<Bytes>, RenderWebsiteError> {
        let (response_sender, response_receiver) = new_response_channel();
        self.request_sender
            .send(WebRendererThreadRequest::GetRenderedWebsite { response_sender })?;

        Ok(response_receiver.recv()?)
    }

    pub fn ensure_shared_memory(&self, sources: &[&NodeTexture]) -> Result<(), RenderWebsiteError> {
        let resolutions = sources.iter().map(|texture| texture.resolution()).collect();
        self.request_sender
            .send(WebRendererThreadRequest::EnsureSharedMemory { resolutions })?;

        Ok(())
    }

    pub fn update_shared_memory(
        &self,
        source_idx: usize,
        buffer: Arc<wgpu::Buffer>,
        size: wgpu::Extent3d,
    ) -> Result<(), RenderWebsiteError> {
        let (response_sender, response_receiver) = new_response_channel();
        let payload = UpdateSharedMemoryPayload {
            source_idx,
            buffer,
            size,
        };

        self.request_sender
            .send(WebRendererThreadRequest::UpdateSharedMemory {
                payload,
                response_sender,
            })?;

        // Wait until buffer unmap is possible
        response_receiver.recv()?;
        Ok(())
    }

    pub fn frame_positions(
        &self,
        children_ids: Vec<ComponentId>,
    ) -> Result<Vec<Position>, RenderWebsiteError> {
        let (response_sender, response_receiver) = new_response_channel();
        self.request_sender
            .send(WebRendererThreadRequest::GetFramePositions {
                children_ids,
                response_sender,
            })?;

        Ok(response_receiver.recv()?)
    }

    fn prepare_textures<'a>(
        &'a self,
        sources: &'a [&NodeTexture],
        frame_positions: &[Position],
    ) -> Vec<(Option<&Texture>, RenderInfo)> {
        let mut source_info = sources
            .iter()
            .zip(frame_positions.iter())
            .map(|(node_texture, position)| {
                (
                    node_texture.texture(),
                    RenderInfo::source_transform(&vertices_transformation_matrix(
                        position,
                        &self.spec.resolution,
                    )),
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

    pub fn resolution(&self) -> Resolution {
        self.spec.resolution
    }

    pub fn shared_memory_root_path(compositor_instance_id: &str, web_renderer_id: &str) -> PathBuf {
        env::temp_dir()
            .join("video_compositor")
            .join(format!("instance_{compositor_instance_id}"))
            .join(web_renderer_id)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateWebRendererError {
    #[error(transparent)]
    CreateShaderFailed(#[from] CreateShaderError),

    #[error("Web rendering can not be used because it was disabled in the init request")]
    WebRendererDisabled,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderWebsiteError {
    #[error("Failed to send request to web renderer thread")]
    WebRendererThreadRequestFailed(#[from] SendError<WebRendererThreadRequest>),

    #[error("Failed to retrieve response from web renderer thread")]
    WebRendererThreadResponseFailed(#[from] ResponseReceiverError),

    #[error("Failed to download source frame")]
    DownloadFrame(#[from] wgpu::BufferAsyncError),
}
