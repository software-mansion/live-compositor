use std::path::PathBuf;
use std::{
    collections::HashMap,
    sync::Arc,
    thread::{self, JoinHandle},
};

use compositor_chromium::cef;
use compositor_common::{
    error::ErrorStack,
    scene::{NodeId, Resolution},
};
use crossbeam_channel::{Receiver, Sender};
use log::error;

use crate::renderer::RegisterCtx;
use crate::transformations::web_renderer::chromium_sender::{
    ChromiumSenderMessage, UpdateSharedMemoryInfo,
};
use crate::transformations::web_renderer::shared_memory::{SharedMemory, SharedMemoryError};
use crate::transformations::web_renderer::WebRenderer;
use crate::{renderer::texture::utils::pad_to_256, EMBED_SOURCES_MESSAGE};

use super::{browser::BrowserClient, chromium_context::ChromiumContext};

pub(super) struct ChromiumSenderThread {
    chromium_ctx: Arc<ChromiumContext>,
    url: String,
    browser_client: BrowserClient,

    message_sender: Sender<ChromiumSenderMessage>,
    message_receiver: Receiver<ChromiumSenderMessage>,
    unmap_signal_sender: Sender<()>,
}

impl ChromiumSenderThread {
    pub fn new(
        ctx: &RegisterCtx,
        url: String,
        mut browser_client: BrowserClient,
        unmap_signal_sender: Sender<()>,
    ) -> Self {
        let (message_sender, message_receiver) = crossbeam_channel::unbounded();
        browser_client.set_chromium_thread_sender(message_sender.clone());

        Self {
            chromium_ctx: ctx.chromium.clone(),
            url,
            browser_client,
            message_sender,
            message_receiver,
            unmap_signal_sender,
        }
    }

    pub fn sender(&self) -> Sender<ChromiumSenderMessage> {
        self.message_sender.clone()
    }

    pub fn spawn(mut self) -> JoinHandle<()> {
        thread::spawn(move || self.run())
    }

    fn run(&mut self) {
        let Ok(browser) = self
            .chromium_ctx
            .start_browser(&self.url, self.browser_client.clone())
        else {
            error!("Couldn't start browser for {}", self.url);
            return;
        };

        let mut state = ThreadState::new(browser, self.chromium_ctx.instance_id());
        loop {
            let result = match self.message_receiver.recv().unwrap() {
                ChromiumSenderMessage::EmbedSources {
                    node_id,
                    resolutions,
                } => Self::embed_frames(&mut state, node_id, resolutions),
                ChromiumSenderMessage::EnsureSharedMemory { node_id, sizes } => {
                    Self::ensure_shared_memory(&mut state, node_id, sizes)
                }
                ChromiumSenderMessage::UpdateSharedMemory(info) => {
                    self.update_shared_memory(&mut state, info)
                }
                ChromiumSenderMessage::ResolveSharedMemoryResize {
                    node_id,
                    source_idx,
                } => Self::resolve_shared_memory_resize(&mut state, node_id, source_idx),
            };

            if let Err(err) = result {
                error!(
                    "Error occurred in chromium sender thread.\n{}",
                    ErrorStack::new(&err).into_string()
                );
            }
        }
    }

    fn embed_frames(
        state: &mut ThreadState,
        node_id: NodeId,
        resolutions: Vec<Option<Resolution>>,
    ) -> Result<(), ChromiumSenderThreadError> {
        let mut process_message = cef::ProcessMessage::new(EMBED_SOURCES_MESSAGE);
        let mut index = 0;

        // IPC message to chromium renderer subprocess consists of:
        // - shared memory path
        // - texture width
        // - texture height
        for (source_idx, resolution) in resolutions.iter().enumerate() {
            let shared_memory = state.shared_memory(&node_id, source_idx)?;
            if !shared_memory.is_accessible() {
                continue;
            }

            let Resolution { width, height } = resolution.unwrap_or_else(|| Resolution {
                width: 0,
                height: 0,
            });
            process_message.write_string(index, shared_memory.to_path_string());
            process_message.write_int(index + 1, source_idx as i32);
            process_message.write_int(index + 2, width as i32);
            process_message.write_int(index + 3, height as i32);

            index += 4;
        }

        let frame = state.browser.main_frame()?;
        frame.send_process_message(cef::ProcessId::Renderer, process_message)?;

        Ok(())
    }

    fn ensure_shared_memory(
        state: &mut ThreadState,
        node_id: NodeId,
        sizes: Vec<usize>,
    ) -> Result<(), ChromiumSenderThreadError> {
        if !state.shared_memory.contains_key(&node_id) {
            state
                .shared_memory
                .insert(node_id.clone(), Vec::with_capacity(sizes.len()));
        }

        // Add missing shared_memory
        let node_shared_memory = state.shared_memory.get_mut(&node_id).unwrap();
        for (source_idx, size) in sizes.iter().enumerate().skip(node_shared_memory.len()) {
            node_shared_memory.push(SharedMemory::new(
                &state.shared_memory_root_path,
                &node_id,
                source_idx,
                *size,
            )?);
        }

        // Ensure existing shared memory
        let frame = state.browser.main_frame()?;
        for (source_idx, size) in sizes.into_iter().enumerate() {
            let shared_memory = &mut node_shared_memory[source_idx];
            if shared_memory.len() != size {
                shared_memory.request_resize(size, &frame)?;
            }
        }

        Ok(())
    }

    fn update_shared_memory(
        &self,
        state: &mut ThreadState,
        info: UpdateSharedMemoryInfo,
    ) -> Result<(), ChromiumSenderThreadError> {
        let shared_memory = state.shared_memory(&info.node_id, info.source_idx)?;
        if !shared_memory.is_accessible() {
            self.unmap_signal_sender.send(()).unwrap();
            return Ok(());
        }

        // Writes buffer data to shared memory
        {
            let range = info.buffer.slice(..).get_mapped_range();
            let chunks = range.chunks((4 * pad_to_256(info.size.width)) as usize);
            for (i, chunk) in chunks.enumerate() {
                let bytes_len = (4 * info.size.width) as usize;
                shared_memory.write(&chunk[..bytes_len], i * bytes_len)?;
            }
        }

        self.unmap_signal_sender.send(()).unwrap();
        Ok(())
    }

    fn resolve_shared_memory_resize(
        state: &mut ThreadState,
        node_id: NodeId,
        source_idx: usize,
    ) -> Result<(), ChromiumSenderThreadError> {
        state
            .shared_memory(&node_id, source_idx)?
            .resolve_pending_resize()
            .map_err(ChromiumSenderThreadError::SharedMemoryError)
    }
}

struct ThreadState {
    browser: cef::Browser,
    shared_memory: HashMap<NodeId, Vec<SharedMemory>>,
    shared_memory_root_path: PathBuf,
}

impl ThreadState {
    fn new(browser: cef::Browser, renderer_id: &str) -> Self {
        let shared_memory_root_path = WebRenderer::shared_memory_root_path(renderer_id);
        let shared_memory = HashMap::new();

        Self {
            browser,
            shared_memory,
            shared_memory_root_path,
        }
    }

    fn shared_memory(
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
}

#[derive(Debug, thiserror::Error)]
enum ChromiumSenderThreadError {
    #[error("Browser is no longer alive")]
    BrowserNotAlive(#[from] cef::BrowserError),

    #[error("Browser frame is no longer alive")]
    FrameNotAlive(#[from] cef::FrameError),

    #[error(transparent)]
    SharedMemoryError(#[from] SharedMemoryError),

    #[error("Shared memory should already be allocated for all inputs of node \"{0}\"")]
    SharedMemoryNotAllocated(NodeId),
}
