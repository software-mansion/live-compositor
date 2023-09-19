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
        let (message_sender, message_receiver) = crossbeam_channel::unbounded();
        let (unmap_signal_sender, unmap_signal_receiver) = crossbeam_channel::bounded(0);

        ChromiumSenderThread::new(
            ctx,
            url,
            browser_client,
            message_receiver,
            unmap_signal_sender,
        )
        .spawn();

        Self {
            message_sender,
            unmap_signal_receiver,
        }
    }

    pub fn embed_sources(&self, node_id: NodeId, sources: &[(&NodeId, &NodeTexture)]) {
        let resolutions = sources
            .iter()
            .filter_map(|(_, texture)| texture.resolution())
            .collect();
        self.message_sender
            .send(ChromiumSenderMessage::EmbedSources {
                node_id,
                resolutions,
            })
            .unwrap();
    }

    pub fn alloc_shared_memory(&self, node_id: NodeId, sources: &[(&NodeId, &NodeTexture)]) {
        let sizes = sources
            .iter()
            .filter_map(|(_, texture)| texture.resolution())
            .map(|res| 4 * res.width * res.height)
            .collect();
        self.message_sender
            .send(ChromiumSenderMessage::AllocSharedMemory { node_id, sizes })
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
        resolutions: Vec<Resolution>,
    },
    AllocSharedMemory {
        node_id: NodeId,
        sizes: Vec<usize>,
    },
    UpdateSharedMemory(UpdateSharedMemoryInfo),
    GetFramePositions {
        source_count: usize,
    },
}

pub(super) struct UpdateSharedMemoryInfo {
    pub node_id: NodeId,
    pub source_idx: usize,
    pub buffer: Arc<wgpu::Buffer>,
    pub size: wgpu::Extent3d,
}
