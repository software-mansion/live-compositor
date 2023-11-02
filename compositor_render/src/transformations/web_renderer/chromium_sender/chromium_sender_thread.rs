use std::{
    path::PathBuf,
    sync::Arc,
    thread::{self, JoinHandle},
};

use compositor_chromium::cef;
use compositor_common::{
    error::ErrorStack,
    scene::{NodeId, Resolution},
};
use crossbeam_channel::{Receiver, Sender};
use log::{error, warn};

use crate::transformations::web_renderer::chromium_sender::{
    ChromiumSenderMessage, UpdateSharedMemoryInfo,
};
use crate::transformations::web_renderer::shared_memory::{SharedMemory, SharedMemoryError};
use crate::{
    renderer::RegisterCtx,
    transformations::web_renderer::{
        browser_client::BrowserClient, chromium_context::ChromiumContext,
    },
};
use crate::{wgpu::texture::utils::pad_to_256, EMBED_SOURCES_MESSAGE, GET_FRAME_POSITIONS_MESSAGE};

use super::thread_state::ThreadState;

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

    pub fn spawn(mut self) -> JoinHandle<()> {
        thread::spawn(move || self.run())
    }

    pub fn sender(&self) -> Sender<ChromiumSenderMessage> {
        self.message_sender.clone()
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
                } => self.embed_frames(&state, node_id, resolutions),
                ChromiumSenderMessage::EnsureSharedMemory { node_id, sizes } => {
                    self.ensure_shared_memory(&mut state, node_id, sizes)
                }
                ChromiumSenderMessage::UpdateSharedMemory(info) => {
                    self.update_shared_memory(&mut state, info)
                }
                ChromiumSenderMessage::GetFramePositions { source_count } => {
                    self.get_frame_positions(&state, source_count)
                }
                ChromiumSenderMessage::FinalizePendingResize { shared_memory_path } => {
                    self.finalize_pending_resize(&mut state, shared_memory_path)
                }
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
        &self,
        state: &ThreadState,
        node_id: NodeId,
        resolutions: Vec<Option<Resolution>>,
    ) -> Result<(), ChromiumSenderThreadError> {
        let Some(shared_memory) = state.shared_memory.get(&node_id) else {
            return Err(ChromiumSenderThreadError::SharedMemoryNotAllocated(node_id));
        };
        let mut process_message = cef::ProcessMessage::new(EMBED_SOURCES_MESSAGE);
        let mut index = 0;

        // IPC message to chromium renderer subprocess consists of:
        // - shared memory path
        // - input/source index
        // - texture width
        // - texture height
        for (source_idx, resolution) in resolutions.iter().enumerate() {
            let shared_memory = &shared_memory[source_idx];
            if !state.is_shared_memory_accessible(shared_memory) {
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

            index += 3;
        }

        let frame = state.browser.main_frame()?;
        frame.send_process_message(cef::ProcessId::Renderer, process_message)?;

        Ok(())
    }

    fn ensure_shared_memory(
        &self,
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
        {
            let node_shared_memory = state.shared_memory.get_mut(&node_id).unwrap();
            for (source_idx, size) in sizes.iter().enumerate().skip(node_shared_memory.len()) {
                node_shared_memory.push(SharedMemory::new(
                    &state.shared_memory_root_path,
                    &node_id,
                    source_idx,
                    *size,
                )?);
            }
        }

        // Ensure existing shared memory
        for (source_idx, size) in sizes.into_iter().enumerate() {
            let shared_memory_len = state.shared_memory(&node_id, source_idx)?.len();
            if shared_memory_len != size {
                state.request_shared_memory_resize(&node_id, source_idx, size)?;
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
        if !state.is_shared_memory_accessible(shared_memory) {
            self.unmap_signal_sender.send(()).unwrap();
            return Ok(());
        }

        // Writes buffer data to shared memory
        {
            let shared_memory = state.shared_memory_mut(&info.node_id, info.source_idx)?;
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

    fn get_frame_positions(
        &self,
        state: &ThreadState,
        source_count: usize,
    ) -> Result<(), ChromiumSenderThreadError> {
        let mut message = cef::ProcessMessage::new(GET_FRAME_POSITIONS_MESSAGE);
        message.write_int(0, source_count as i32);

        let frame = state.browser.main_frame()?;
        frame.send_process_message(cef::ProcessId::Renderer, message)?;

        Ok(())
    }

    fn finalize_pending_resize(
        &self,
        state: &mut ThreadState,
        shared_memory_path: PathBuf,
    ) -> Result<(), ChromiumSenderThreadError> {
        let Some(pending_resize) = state.pending_resizes.remove(&shared_memory_path) else {
            warn!("There is no pending resize for \"{shared_memory_path:?}\"");
            return Ok(());
        };

        let shared_memory =
            state.shared_memory_mut(&pending_resize.node_id, pending_resize.source_idx)?;
        shared_memory.resize(pending_resize.new_size)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChromiumSenderThreadError {
    #[error("Browser is no longer alive")]
    BrowserNotAlive(#[from] cef::BrowserError),

    #[error("Browser frame is no longer alive")]
    FrameNotAlive(#[from] cef::FrameError),

    #[error(transparent)]
    SharedMemoryError(#[from] SharedMemoryError),

    #[error("Shared memory should already be allocated for all inputs of node \"{0}\"")]
    SharedMemoryNotAllocated(NodeId),
}
