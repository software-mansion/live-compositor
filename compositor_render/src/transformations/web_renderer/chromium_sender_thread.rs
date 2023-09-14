use std::{
    collections::HashMap,
    env,
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
use shared_memory::{Shmem, ShmemConf, ShmemError};

use crate::transformations::web_renderer::chromium_sender::ChromiumSenderMessage;
use crate::{
    renderer::texture::utils::pad_to_256, EMBED_SOURCE_FRAMES_MESSAGE,
    UNEMBED_SOURCE_FRAMES_MESSAGE,
};

use super::{browser::BrowserClient, chromium_context::ChromiumContext};

pub(super) struct ChromiumSenderThread {
    ctx: Arc<ChromiumContext>,
    url: String,
    browser_client: BrowserClient,

    message_receiver: Receiver<ChromiumSenderMessage>,
    unmap_signal_sender: Sender<()>,
}

impl ChromiumSenderThread {
    pub fn new(
        ctx: Arc<ChromiumContext>,
        url: String,
        browser_client: BrowserClient,
        message_receiver: Receiver<ChromiumSenderMessage>,
        unmap_signal_sender: Sender<()>,
    ) -> Self {
        Self {
            ctx,
            url,
            browser_client,
            message_receiver,
            unmap_signal_sender,
        }
    }

    pub fn spawn(mut self) -> JoinHandle<()> {
        thread::spawn(move || self.run())
    }

    fn run(&mut self) {
        let Ok(browser) = self
            .ctx
            .start_browser(&self.url, self.browser_client.clone())
        else {
            error!("Couldn't start browser for {}", self.url);
            return;
        };

        let mut state = ThreadState {
            browser,
            shared_memories: HashMap::new(),
        };

        loop {
            let result = match self.message_receiver.recv().unwrap() {
                ChromiumSenderMessage::EmbedSources {
                    node_id,
                    resolutions,
                } => self.handle_embed_frames(&state, node_id, resolutions),
                ChromiumSenderMessage::UpdateSharedMemory {
                    node_id,
                    source_idx,
                    buffer,
                    size,
                } => self.handle_shmem_update(&mut state, node_id, source_idx, buffer, size),
            };

            if let Err(err) = result {
                error!(
                    "Error occurred in chromium sender thread.\n{}",
                    ErrorStack::new(&err).into_string()
                );
            }
        }
    }

    fn handle_embed_frames(
        &self,
        state: &ThreadState,
        node_id: NodeId,
        resolutions: Vec<Resolution>,
    ) -> Result<(), ChromiumSenderThreadError> {
        let mut process_message = cef::ProcessMessage::new(EMBED_SOURCE_FRAMES_MESSAGE);
        let mut index = 0;

        // IPC message to chromium renderer subprocess consists of:
        // - web renderer node ID - used as a shared memory folder name
        // - input node index - used for retrieving frame data from shared memory and it also serves as variable name in JS
        // - texture width
        // - texture height
        process_message.write_string(index, node_id.to_string());
        index += 1;
        for (i, resolution) in resolutions.iter().enumerate() {
            process_message.write_int(index, i as i32);
            process_message.write_int(index + 1, resolution.width as i32);
            process_message.write_int(index + 2, resolution.height as i32);
            index += 3;
        }

        let frame = state.browser.main_frame()?;
        frame.send_process_message(cef::ProcessId::Renderer, process_message)?;

        Ok(())
    }

    // TODO: Synchronize shared memory access
    fn handle_shmem_update(
        &self,
        state: &mut ThreadState,
        node_id: NodeId,
        source_idx: usize,
        buffer: Arc<wgpu::Buffer>,
        size: wgpu::Extent3d,
    ) -> Result<(), ChromiumSenderThreadError> {
        let shmem = match state.shared_memories.get(&(node_id.clone(), source_idx)) {
            Some(shmem) => {
                if shmem.len() != (4 * size.width * size.height) as usize {
                    state.resize_shmem((node_id.clone(), source_idx), size)?
                } else {
                    shmem
                }
            }
            None => state.create_insert_shmem((node_id.clone(), source_idx), size)?,
        };

        // Writes buffer data to shared memory
        {
            let shmem = shmem.as_ptr();
            let range = buffer.slice(..).get_mapped_range();
            let chunks = range.chunks((4 * pad_to_256(size.width)) as usize);
            for (i, chunk) in chunks.enumerate() {
                unsafe {
                    std::ptr::copy(
                        chunk.as_ptr(),
                        shmem.add(i * 4 * size.width as usize),
                        4 * size.width as usize,
                    )
                }
            }
        }

        self.unmap_signal_sender.send(()).unwrap();
        Ok(())
    }
}

struct ThreadState {
    browser: cef::Browser,
    shared_memories: HashMap<(NodeId, usize), Shmem>,
}

impl ThreadState {
    fn create_insert_shmem(
        &mut self,
        key: (NodeId, usize),
        size: wgpu::Extent3d,
    ) -> Result<&Shmem, ChromiumSenderThreadError> {
        let shmem_path = env::temp_dir()
            .join(key.0.to_string())
            .join(key.1.to_string());
        let shmem = ShmemConf::new()
            .flink(shmem_path)
            .size((4 * size.width * size.height) as usize)
            .force_create_flink()
            .create()?;

        self.shared_memories.insert(key.clone(), shmem);
        Ok(self.shared_memories.get(&key).unwrap())
    }

    fn resize_shmem(
        &mut self,
        key: (NodeId, usize),
        size: wgpu::Extent3d,
    ) -> Result<&Shmem, ChromiumSenderThreadError> {
        let mut process_message = cef::ProcessMessage::new(UNEMBED_SOURCE_FRAMES_MESSAGE);
        process_message.write_string(0, key.0.to_string());
        process_message.write_int(1, key.1 as i32);

        let frame = self.browser.main_frame()?;
        frame.send_process_message(cef::ProcessId::Renderer, process_message)?;

        self.shared_memories.remove(&key);
        self.create_insert_shmem(key, size)
    }
}

#[derive(Debug, thiserror::Error)]
enum ChromiumSenderThreadError {
    #[error("Browser is no longer alive")]
    BrowserNotAlive(#[from] cef::BrowserError),

    #[error("Browser frame is no longer alive")]
    FrameNotAlive(#[from] cef::FrameError),

    #[error("Failed to create shared memory")]
    CreateShmemFailed(#[from] ShmemError),
}
