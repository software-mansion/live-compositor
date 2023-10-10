use std::sync::{Arc, Mutex};

use bytes::Bytes;
use compositor_chromium::cef;
use compositor_common::scene::{NodeId, Resolution};
use crossbeam_channel::{bounded, Sender};
use log::error;
use shared_memory::ShmemError;

use crate::{
    renderer::{texture::NodeTexture, RegisterCtx, RenderCtx},
    UNEMBED_SOURCE_MESSAGE,
};

use super::chromium_sender::{ChromiumSender, ChromiumSenderMessage};

pub(super) struct BrowserController {
    chromium_sender: ChromiumSender,
    frame_data: Arc<Mutex<Bytes>>,
}

impl BrowserController {
    pub fn new(ctx: &RegisterCtx, url: String, resolution: Resolution) -> Self {
        let frame_data = Arc::new(Mutex::new(Bytes::new()));
        let client = BrowserClient::new(frame_data.clone(), resolution);
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
            .ensure_shared_memory(node_id.clone(), sources);
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
    resolution: Resolution,
    chromium_thread_sender: Option<Sender<ChromiumSenderMessage>>,
}

impl cef::Client for BrowserClient {
    type RenderHandlerType = RenderHandler;

    fn render_handler(&self) -> Option<Self::RenderHandlerType> {
        Some(RenderHandler::new(self.frame_data.clone(), self.resolution))
    }

    fn on_process_message_received(
        &mut self,
        _browser: &cef::Browser,
        _frame: &cef::Frame,
        _source_process: cef::ProcessId,
        message: &cef::ProcessMessage,
    ) -> bool {
        match message.name().as_str() {
            UNEMBED_SOURCE_MESSAGE => self.resolve_shared_memory_resize(message),
            ty => error!("Unknown process message type \"{ty}\""),
        }

        // Message handled
        true
    }
}

impl BrowserClient {
    pub fn new(frame_data: Arc<Mutex<Bytes>>, resolution: Resolution) -> Self {
        Self {
            frame_data,
            resolution,
            chromium_thread_sender: None,
        }
    }

    pub fn set_chromium_thread_sender(&mut self, sender: Sender<ChromiumSenderMessage>) {
        self.chromium_thread_sender.replace(sender);
    }

    fn resolve_shared_memory_resize(&mut self, message: &cef::ProcessMessage) {
        let Some(node_id) = message.read_string(0) else {
            error!("Failed to read node_id");
            return;
        };
        let Some(source_idx) = message.read_int(1) else {
            error!("Failed to read source index of shared memory");
            return;
        };

        if let Some(chromium_sender) = &self.chromium_thread_sender {
            chromium_sender
                .send(ChromiumSenderMessage::ResolveSharedMemoryResize {
                    node_id: NodeId(node_id.into()),
                    source_idx: source_idx as usize,
                })
                .unwrap();
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
