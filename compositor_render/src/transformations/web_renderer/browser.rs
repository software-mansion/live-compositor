use std::sync::{Arc, Mutex};

use crate::GET_FRAME_POSITIONS_MESSAGE;
use bytes::{Bytes, BytesMut};
use compositor_chromium::cef;
use compositor_chromium::cef::{Browser, Frame, ProcessId, ProcessMessage};
use compositor_common::scene::{NodeId, Resolution};
use crossbeam_channel::bounded;
use log::error;
use shared_memory::ShmemError;

use crate::renderer::{texture::NodeTexture, RegisterCtx, RenderCtx};
use crate::transformations::builtin::box_layout::BoxLayout;
use crate::transformations::builtin::utils::mat4_to_bytes;
use crate::transformations::web_renderer::frame_embedder::FrameTransforms;

use super::chromium_sender::ChromiumSender;

pub(super) struct BrowserController {
    chromium_sender: ChromiumSender,
    frame_data: Arc<Mutex<Bytes>>,
}

impl BrowserController {
    pub fn new(
        ctx: &RegisterCtx,
        url: String,
        resolution: Resolution,
        frame_positions: FrameTransforms,
    ) -> Self {
        let frame_data = Arc::new(Mutex::new(Bytes::new()));
        let client = BrowserClient::new(frame_data.clone(), frame_positions, resolution);
        let chromium_sender = ChromiumSender::new(ctx, url, client);

        Self {
            chromium_sender,
            frame_data,
        }
    }

    pub fn retrieve_frame(&mut self) -> Option<Bytes> {
        let frame_data = self.frame_data.lock().unwrap();
        if frame_data.is_empty() {
            return None;
        }
        Some(frame_data.clone())
    }

    pub fn send_sources(
        &mut self,
        ctx: &RenderCtx,
        node_id: NodeId,
        sources: &[(&NodeId, &NodeTexture)],
        buffers: &[Arc<wgpu::Buffer>],
    ) -> Result<(), EmbedFrameError> {
        self.chromium_sender
            .alloc_shared_memory(node_id.clone(), sources);
        self.copy_sources_to_buffers(ctx, sources, buffers)?;

        let mut pending_downloads = Vec::new();
        for (source_idx, ((_, texture), buffer)) in sources.iter().zip(buffers).enumerate() {
            let Some(texture_state) = texture.state() else {
                continue;
            };
            let size = texture_state.rgba_texture().size();
            pending_downloads.push(self.copy_buffer_to_shmem(
                node_id.clone(),
                source_idx,
                size,
                buffer.clone(),
            ));
        }

        ctx.wgpu_ctx.device.poll(wgpu::Maintain::Wait);

        for pending in pending_downloads {
            pending()?;
        }

        self.chromium_sender.embed_sources(node_id, sources);
        Ok(())
    }

    pub fn request_frame_positions(&self, sources: &[(&NodeId, &NodeTexture)]) {
        self.chromium_sender.request_frame_positions(sources);
    }

    fn copy_sources_to_buffers(
        &self,
        ctx: &RenderCtx,
        sources: &[(&NodeId, &NodeTexture)],
        buffers: &[Arc<wgpu::Buffer>],
    ) -> Result<(), EmbedFrameError> {
        let mut encoder = ctx
            .wgpu_ctx
            .device
            .create_command_encoder(&Default::default());

        for ((_, texture), buffer) in sources.iter().zip(buffers) {
            let Some(texture_state) = texture.state() else {
                continue;
            };
            texture_state
                .rgba_texture()
                .copy_to_buffer(&mut encoder, buffer);
        }
        ctx.wgpu_ctx.queue.submit(Some(encoder.finish()));

        Ok(())
    }

    fn copy_buffer_to_shmem(
        &self,
        node_id: NodeId,
        source_idx: usize,
        size: wgpu::Extent3d,
        source: Arc<wgpu::Buffer>,
    ) -> impl FnOnce() -> Result<(), EmbedFrameError> + '_ {
        let (s, r) = bounded(1);
        source
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                if let Err(err) = s.send(result) {
                    error!("channel send error: {err}")
                }
            });

        move || {
            r.recv().unwrap()?;

            self.chromium_sender
                .update_shared_memory(node_id, source_idx, source.clone(), size);
            source.unmap();

            Ok(())
        }
    }
}

#[derive(Clone)]
pub(super) struct BrowserClient {
    frame_data: Arc<Mutex<Bytes>>,
    frame_positions: FrameTransforms,
    resolution: Resolution,
}

impl cef::Client for BrowserClient {
    type RenderHandlerType = RenderHandler;

    fn render_handler(&self) -> Option<Self::RenderHandlerType> {
        Some(RenderHandler::new(self.frame_data.clone(), self.resolution))
    }

    fn on_process_message_received(
        &mut self,
        _browser: &Browser,
        _frame: &Frame,
        _source_process: ProcessId,
        message: &ProcessMessage,
    ) -> bool {
        match message.name().as_str() {
            GET_FRAME_POSITIONS_MESSAGE => {
                let mut transformations_matrices = BytesMut::new();
                for i in (0..message.size()).step_by(4) {
                    let Some(x) = message.read_double(i) else {
                        error!("Expected \"x\" value of DOMRect");
                        continue;
                    };
                    let Some(y) = message.read_double(i + 1) else {
                        error!("Expected \"y\" value of DOMRect");
                        continue;
                    };
                    let Some(width) = message.read_double(i + 2) else {
                        error!("Expected \"width\" value of DOMRect");
                        continue;
                    };
                    let Some(height) = message.read_double(i + 3) else {
                        error!("Expected \"height\" value of DOMRect");
                        continue;
                    };

                    let transformations_matrix = BoxLayout {
                        top_left_corner: (x as f32, y as f32),
                        width: width as f32,
                        height: height as f32,
                        rotation_degrees: 0.0,
                    }
                    .transformation_matrix(self.resolution);

                    transformations_matrices.extend(mat4_to_bytes(&transformations_matrix));
                }

                let mut frame_positions = self.frame_positions.lock().unwrap();
                *frame_positions = transformations_matrices.freeze();
            }
            ty => error!("Unknown process message type \"{ty}\""),
        }
        true
    }
}

impl BrowserClient {
    pub fn new(
        frame_data: Arc<Mutex<Bytes>>,
        frame_positions: FrameTransforms,
        resolution: Resolution,
    ) -> Self {
        Self {
            frame_data,
            frame_positions,
            resolution,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EmbedFrameError {
    #[error("Failed to create shared memory")]
    CreateSharedMemory(#[from] ShmemError),

    #[error("Failed to download source frame")]
    DownloadFrame(#[from] wgpu::BufferAsyncError),

    #[error("Browser is no longer alive")]
    BrowserNotAlive(#[from] cef::BrowserError),

    #[error("Could not send IPC message")]
    MessageNotSent(#[from] cef::FrameError),

    #[error("Download buffer does not exist")]
    ExpectDownloadBuffer,
}

pub(super) struct RenderHandler {
    frame_data: Arc<Mutex<Bytes>>,
    resolution: Resolution,
}

impl cef::RenderHandler for RenderHandler {
    fn resolution(&self, _browser: &cef::Browser) -> Resolution {
        self.resolution
    }

    fn on_paint(&self, _browser: &cef::Browser, buffer: &[u8], _resolution: Resolution) {
        let mut frame_data = self.frame_data.lock().unwrap();
        *frame_data = Bytes::copy_from_slice(buffer);
    }
}

impl RenderHandler {
    pub fn new(frame_data: Arc<Mutex<Bytes>>, resolution: Resolution) -> Self {
        Self {
            frame_data,
            resolution,
        }
    }
}
