use std::{collections::HashMap, path::PathBuf};

use compositor_chromium::cef;
use compositor_common::scene::NodeId;

use crate::{
    transformations::web_renderer::{shared_memory::SharedMemory, WebRenderer},
    UNEMBED_SOURCE_MESSAGE,
};

use super::chromium_sender_thread::ChromiumSenderThreadError;

pub(super) struct ThreadState {
    pub browser: cef::Browser,
    pub shared_memory: HashMap<NodeId, Vec<SharedMemory>>,
    pub shared_memory_root_path: PathBuf,
    pub pending_resizes: HashMap<PathBuf, PendingResize>,
}

impl ThreadState {
    pub fn new(browser: cef::Browser, renderer_id: &str) -> Self {
        let shared_memory_root_path = WebRenderer::shared_memory_root_path(renderer_id);
        let shared_memory = HashMap::new();
        let pending_resizes = HashMap::new();

        Self {
            browser,
            shared_memory,
            shared_memory_root_path,
            pending_resizes,
        }
    }

    pub fn shared_memory(
        &self,
        node_id: &NodeId,
        source_idx: usize,
    ) -> Result<&SharedMemory, ChromiumSenderThreadError> {
        let node_shared_memory = self
            .shared_memory
            .get(node_id)
            .ok_or_else(|| ChromiumSenderThreadError::SharedMemoryNotAllocated(node_id.clone()))?;

        Ok(&node_shared_memory[source_idx])
    }

    pub fn shared_memory_mut(
        &mut self,
        node_id: &NodeId,
        source_idx: usize,
    ) -> Result<&mut SharedMemory, ChromiumSenderThreadError> {
        let node_shared_memory = self
            .shared_memory
            .get_mut(node_id)
            .ok_or_else(|| ChromiumSenderThreadError::SharedMemoryNotAllocated(node_id.clone()))?;

        Ok(&mut node_shared_memory[source_idx])
    }

    pub fn request_shared_memory_resize(
        &mut self,
        node_id: &NodeId,
        source_idx: usize,
        new_size: usize,
    ) -> Result<(), ChromiumSenderThreadError> {
        let shared_memory_path = self.shared_memory(node_id, source_idx)?.path().to_owned();
        if let Some(pending_resize) = self.pending_resizes.get_mut(&shared_memory_path) {
            pending_resize.new_size = new_size;
            // UNEMBED_SOURCE request is already sent by the previous resize request
            return Ok(());
        }

        self.pending_resizes.insert(
            shared_memory_path.clone(),
            PendingResize {
                node_id: node_id.clone(),
                source_idx,
                new_size,
            },
        );
        let frame = self.browser.main_frame()?;
        let mut msg = cef::ProcessMessage::new(UNEMBED_SOURCE_MESSAGE);
        msg.write_string(0, shared_memory_path.display().to_string());
        frame.send_process_message(cef::ProcessId::Renderer, msg)?;

        Ok(())
    }

    pub fn is_shared_memory_accessible(&self, shared_memory: &SharedMemory) -> bool {
        !self.pending_resizes.contains_key(shared_memory.path())
    }
}

pub(super) struct PendingResize {
    pub node_id: NodeId,
    pub source_idx: usize,
    pub new_size: usize,
}
