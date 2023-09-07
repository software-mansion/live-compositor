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
use crossbeam_channel::{select, Receiver, Sender};
use log::error;
use shared_memory::{Shmem, ShmemConf, ShmemError};

use crate::{
    renderer::texture::utils::pad_to_256, EMBED_SOURCE_FRAMES_MESSAGE,
    UNEMBED_SOURCE_FRAMES_MESSAGE,
};

use super::{browser::BrowserClient, chromium_context::ChromiumContext};

pub(super) struct ChromiumSenderThread {
    ctx: Arc<ChromiumContext>,
    url: String,
    browser_client: BrowserClient,

    embed_sources_receiver: Receiver<Vec<(NodeId, Resolution)>>,
    shmem_update_receiver: Receiver<(NodeId, Arc<wgpu::Buffer>, wgpu::Extent3d)>,
    unmap_signal_sender: Sender<()>,
}

impl ChromiumSenderThread {
    pub fn new(
        ctx: Arc<ChromiumContext>,
        url: String,
        browser_client: BrowserClient,
        embed_sources_receiver: Receiver<Vec<(NodeId, Resolution)>>,
        shmem_update_receiver: Receiver<(NodeId, Arc<wgpu::Buffer>, wgpu::Extent3d)>,
        unmap_signal_sender: Sender<()>,
    ) -> Self {
        Self {
            ctx,
            url,
            browser_client,
            embed_sources_receiver,
            shmem_update_receiver,
            unmap_signal_sender,
        }
    }

    pub fn spawn(mut self) -> JoinHandle<()> {
        thread::spawn(move || self.run())
    }

    fn run(&mut self) {
        let mut shared_memories = HashMap::new();
        let Ok(browser) = self
            .ctx
            .start_browser(&self.url, self.browser_client.clone())
        else {
            error!("Couldn't start browser for {}", self.url);
            return;
        };

        loop {
            let result = select! {
                recv(self.embed_sources_receiver) -> sources_info => self.handle_embed_frames(&browser, sources_info.unwrap()),
                recv(self.shmem_update_receiver) -> data => self.handle_shmem_update(&browser, &mut shared_memories, data.unwrap()),
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
        browser: &cef::Browser,
        sources_info: Vec<(NodeId, Resolution)>,
    ) -> Result<(), ChromiumSenderThreadError> {
        let mut process_message = cef::ProcessMessage::new(EMBED_SOURCE_FRAMES_MESSAGE);
        let mut index = 0;

        // IPC message to chromium renderer subprocess consists of:
        // - input node ID - used for retrieving frame data from shared memory and it also serves as variable name in JS
        // - texture width
        // - texture height
        for (id, resolution) in sources_info {
            process_message.write_string(index, id.to_string());
            process_message.write_int(index + 1, resolution.width as i32);
            process_message.write_int(index + 2, resolution.height as i32);
            index += 3;
        }

        let frame = browser.main_frame()?;
        frame.send_process_message(cef::ProcessId::Renderer, process_message)?;

        Ok(())
    }

    // TODO: Synchronize shared memory access
    fn handle_shmem_update(
        &mut self,
        browser: &cef::Browser,
        shared_memories: &mut HashMap<NodeId, Shmem>,
        (id, buffer, size): (NodeId, Arc<wgpu::Buffer>, wgpu::Extent3d),
    ) -> Result<(), ChromiumSenderThreadError> {
        let shmem = match shared_memories.get(&id) {
            Some(shmem) => {
                if shmem.len() != (4 * size.width * size.height) as usize {
                    self.resize_shmem(id, size, browser, shared_memories)?
                } else {
                    shmem
                }
            }
            None => self.create_insert_shmem(id, size, shared_memories)?,
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

    fn create_insert_shmem<'a>(
        &'a mut self,
        source_id: NodeId,
        size: wgpu::Extent3d,
        shared_memories: &'a mut HashMap<NodeId, Shmem>,
    ) -> Result<&Shmem, ChromiumSenderThreadError> {
        let shmem = ShmemConf::new()
            .flink(env::temp_dir().join(source_id.to_string()))
            .size((4 * size.width * size.height) as usize)
            .force_create_flink()
            .create()?;
        shared_memories.insert(source_id.clone(), shmem);
        Ok(shared_memories.get(&source_id).unwrap())
    }

    fn resize_shmem<'a>(
        &'a mut self,
        source_id: NodeId,
        size: wgpu::Extent3d,
        browser: &cef::Browser,
        shared_memories: &'a mut HashMap<NodeId, Shmem>,
    ) -> Result<&Shmem, ChromiumSenderThreadError> {
        let mut process_message = cef::ProcessMessage::new(UNEMBED_SOURCE_FRAMES_MESSAGE);
        process_message.write_string(0, source_id.to_string());

        let frame = browser.main_frame()?;
        frame.send_process_message(cef::ProcessId::Renderer, process_message)?;

        shared_memories.remove(&source_id);
        self.create_insert_shmem(source_id, size, shared_memories)
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
