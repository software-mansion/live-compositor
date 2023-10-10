use std::sync::Arc;

use crate::renderer::RegisterCtx;
use compositor_common::scene::{NodeId, Resolution};
use crossbeam_channel::{Receiver, Sender};

use crate::renderer::texture::NodeTexture;

use super::{browser::BrowserClient, chromium_sender_thread::ChromiumSenderThread};

pub(super) struct ChromiumSender {
    message_sender: Sender<ChromiumSenderMessage>,
    /// Used for synchronizing buffer map and unmap operations
    unmap_signal_receiver: Receiver<()>,
}

impl ChromiumSender {
    pub fn new(ctx: &RegisterCtx, url: String, browser_client: BrowserClient) -> Self {
        let (unmap_signal_sender, unmap_signal_receiver) = crossbeam_channel::bounded(0);
        let chromium_thread =
            ChromiumSenderThread::new(ctx, url, browser_client, unmap_signal_sender);
        let message_sender = chromium_thread.sender();

        chromium_thread.spawn();

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
    ResolveSharedMemoryResize {
        node_id: NodeId,
        source_idx: usize,
    },
}

pub(super) struct UpdateSharedMemoryInfo {
    pub node_id: NodeId,
    pub source_idx: usize,
    pub buffer: Arc<wgpu::Buffer>,
    pub size: wgpu::Extent3d,
}
