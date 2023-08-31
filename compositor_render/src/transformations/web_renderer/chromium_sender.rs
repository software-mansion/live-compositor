use std::sync::Arc;

use compositor_common::scene::{NodeId, Resolution};
use crossbeam_channel::{Receiver, Sender};

use crate::renderer::texture::NodeTexture;

use super::{
    browser::BrowserClient, chromium_context::ChromiumContext,
    chromium_sender_thread::ChromiumSenderThread,
};

pub(super) struct ChromiumSender {
    embed_sources_sender: Sender<Vec<(NodeId, Resolution)>>,
    shmem_update_sender: Sender<(NodeId, Arc<wgpu::Buffer>, wgpu::Extent3d)>,
    /// Used for synchronizing buffer map and unmap operations
    unmap_signal_receiver: Receiver<()>,
}

impl ChromiumSender {
    pub fn new(ctx: Arc<ChromiumContext>, url: String, browser_client: BrowserClient) -> Self {
        let (embed_sources_sender, embed_sources_receiver) = crossbeam_channel::unbounded();
        let (shmem_update_sender, shmem_update_receiver) = crossbeam_channel::unbounded();
        let (unmap_signal_sender, unmap_signal_receiver) = crossbeam_channel::bounded(0);

        ChromiumSenderThread::new(
            ctx,
            url,
            browser_client,
            embed_sources_receiver,
            shmem_update_receiver,
            unmap_signal_sender,
        )
        .spawn();

        Self {
            embed_sources_sender,
            shmem_update_sender,
            unmap_signal_receiver,
        }
    }

    pub fn embed_sources(&self, sources: &[(&NodeId, &NodeTexture)]) {
        let sources_info = sources
            .iter()
            .filter_map(|(id, texture)| texture.resolution().map(|res| ((*id).clone(), res)))
            .collect();
        self.embed_sources_sender.send(sources_info).unwrap();
    }

    pub fn update_shared_memory(
        &self,
        source_id: NodeId,
        buffer: Arc<wgpu::Buffer>,
        size: wgpu::Extent3d,
    ) {
        self.shmem_update_sender
            .send((source_id, buffer, size))
            .unwrap();

        // Wait until buffer unmap is possible
        self.unmap_signal_receiver.recv().unwrap();
    }
}
