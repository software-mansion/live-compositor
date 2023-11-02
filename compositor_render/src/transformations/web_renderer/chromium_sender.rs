use std::{path::PathBuf, sync::Arc};

use crate::renderer::RegisterCtx;
use compositor_common::scene::{NodeId, Resolution};
use crossbeam_channel::{Receiver, Sender};

use crate::wgpu::texture::NodeTexture;

use self::chromium_sender_thread::ChromiumSenderThread;

use super::browser_client::BrowserClient;

mod chromium_sender_thread;
mod thread_state;

pub(super) struct ChromiumSender {
    message_sender: Sender<ChromiumSenderMessage>,
    /// Used for synchronizing buffer map and unmap operations
    unmap_signal_receiver: Receiver<()>,
}

impl ChromiumSender {
    pub fn new(ctx: &RegisterCtx, url: String, browser_client: BrowserClient) -> Self {
        let (unmap_signal_sender, unmap_signal_receiver) = crossbeam_channel::bounded(0);
        let chromium_sender_thread =
            ChromiumSenderThread::new(ctx, url, browser_client, unmap_signal_sender);
        let message_sender = chromium_sender_thread.sender();

        chromium_sender_thread.spawn();

        Self {
            message_sender,
            unmap_signal_receiver,
        }
    }

    pub fn embed_sources(&self, node_id: NodeId, sources: &[(&NodeId, &NodeTexture)]) {
        let resolutions = sources
            .iter()
            .map(|(_, texture)| texture.resolution())
            .collect();
        self.message_sender
            .send(ChromiumSenderMessage::EmbedSources {
                node_id,
                resolutions,
            })
            .unwrap();
    }

    pub fn ensure_shared_memory(&self, node_id: NodeId, sources: &[(&NodeId, &NodeTexture)]) {
        let sizes = sources
            .iter()
            .map(|(_, texture)| {
                texture
                    .resolution()
                    .map(|res| 4 * res.width * res.height)
                    .unwrap_or_default()
            })
            .collect();

        self.message_sender
            .send(ChromiumSenderMessage::EnsureSharedMemory { node_id, sizes })
            .unwrap();
    }

    pub fn update_shared_memory(
        &self,
        node_id: NodeId,
        source_idx: usize,
        buffer: Arc<wgpu::Buffer>,
        size: wgpu::Extent3d,
    ) {
        let info = UpdateSharedMemoryInfo {
            node_id,
            source_idx,
            buffer,
            size,
        };

        self.message_sender
            .send(ChromiumSenderMessage::UpdateSharedMemory(info))
            .unwrap();

        // Wait until buffer unmap is possible
        self.unmap_signal_receiver.recv().unwrap();
    }

    pub fn request_frame_positions(&self, sources: &[(&NodeId, &NodeTexture)]) {
        self.message_sender
            .send(ChromiumSenderMessage::GetFramePositions {
                source_count: sources.len(),
            })
            .unwrap();
    }
}

pub(super) enum ChromiumSenderMessage {
    EmbedSources {
        node_id: NodeId,
        resolutions: Vec<Option<Resolution>>,
    },
    EnsureSharedMemory {
        node_id: NodeId,
        sizes: Vec<usize>,
    },
    UpdateSharedMemory(UpdateSharedMemoryInfo),
    GetFramePositions {
        source_count: usize,
    },
    FinalizePendingResize {
        shared_memory_path: PathBuf,
    },
}

pub(super) struct UpdateSharedMemoryInfo {
    pub node_id: NodeId,
    pub source_idx: usize,
    pub buffer: Arc<wgpu::Buffer>,
    pub size: wgpu::Extent3d,
}
