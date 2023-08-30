use std::{collections::HashMap, path::PathBuf, sync::Arc, thread};

use compositor_chromium::cef;
use compositor_common::scene::{NodeId, Resolution};
use crossbeam_channel::{select, Receiver, Sender};
use log::error;
use shared_memory::{Shmem, ShmemConf, ShmemError};

use crate::{
    renderer::texture::{utils::pad_to_256, NodeTexture},
    SHMEM_FOLDER_PATH, UNEMBED_SOURCE_FRAMES_MESSAGE,
};

use super::{browser::BrowserClient, chromium::ChromiumContext, EMBED_SOURCE_FRAMES_MESSAGE};

/// Handles structs which are not `Sync` and `Send`
pub(super) struct NonSyncSendHandler {
    embed_sources_sender: Sender<Vec<(NodeId, Resolution)>>,
    shmem_update_sender: Sender<(NodeId, Arc<wgpu::Buffer>, wgpu::Extent3d)>,
    /// Used for synchronizing buffer map and unmap operations
    unmap_signal_receiver: Receiver<()>,
}

impl NonSyncSendHandler {
    pub fn new(ctx: Arc<ChromiumContext>, url: String, client: BrowserClient) -> Self {
        let (embed_sources_sender, embed_sources_receiver) = crossbeam_channel::unbounded();
        let (shmem_update_sender, shmem_update_receiver) = crossbeam_channel::unbounded();
        let (unmap_signal_sender, unmap_signal_receiver) = crossbeam_channel::bounded(0);

        thread::spawn(move || {
            let shared_memories = HashMap::new();
            let Ok(browser) = ctx.start_browser(&url, client) else {
                error!("Couldn't start browser for {url}");
                return;
            };
            let mut handler = InnerHandler {
                browser,
                shared_memories,
                unmap_signal_sender,
            };

            loop {
                let result = select! {
                    recv(embed_sources_receiver) -> sources_info => handler.handle_embed_frames(sources_info.unwrap()),
                    recv(shmem_update_receiver) -> data => handler.handle_shmem_update(data.unwrap()),
                };

                if let Err(err) = result {
                    error!("Error occurred in inner handler: {err}");
                }
            }
        });

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

struct InnerHandler {
    browser: cef::Browser,
    shared_memories: HashMap<NodeId, Shmem>,
    unmap_signal_sender: Sender<()>,
}

impl InnerHandler {
    fn handle_embed_frames(
        &self,
        sources_info: Vec<(NodeId, Resolution)>,
    ) -> Result<(), InnerHandlerError> {
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

        let frame = self.browser.main_frame()?;
        frame.send_process_message(cef::ProcessId::Renderer, process_message)?;

        Ok(())
    }

    fn handle_shmem_update(
        &mut self,
        (id, buffer, size): (NodeId, Arc<wgpu::Buffer>, wgpu::Extent3d),
    ) -> Result<(), InnerHandlerError> {
        let shmem = match self.shared_memories.get(&id) {
            Some(shmem) => {
                if shmem.len() != (4 * size.width * size.height) as usize {
                    self.resize_shmem(id, size)?
                } else {
                    shmem
                }
            }
            None => self.create_insert_shmem(id, size)?,
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

    fn create_insert_shmem(
        &mut self,
        source_id: NodeId,
        size: wgpu::Extent3d,
    ) -> Result<&Shmem, InnerHandlerError> {
        let shmem = ShmemConf::new()
            .flink(PathBuf::from(SHMEM_FOLDER_PATH).join(source_id.to_string()))
            .size((4 * size.width * size.height) as usize)
            .force_create_flink()
            .create()?;
        self.shared_memories.insert(source_id.clone(), shmem);
        Ok(self.shared_memories.get(&source_id).unwrap())
    }

    fn resize_shmem(
        &mut self,
        source_id: NodeId,
        size: wgpu::Extent3d,
    ) -> Result<&Shmem, InnerHandlerError> {
        let mut process_message = cef::ProcessMessage::new(UNEMBED_SOURCE_FRAMES_MESSAGE);
        process_message.write_string(0, source_id.to_string());

        let frame = self.browser.main_frame()?;
        frame.send_process_message(cef::ProcessId::Renderer, process_message)?;

        self.shared_memories.remove(&source_id);
        self.create_insert_shmem(source_id, size)
    }
}

#[derive(Debug, thiserror::Error)]
enum InnerHandlerError {
    #[error("Browser is no longer alive")]
    BrowserNotAlive(#[from] cef::BrowserError),

    #[error("Browser frame is no longer alive")]
    FrameNotAlive(#[from] cef::FrameError),

    #[error("Failed to create shared memory")]
    ShmemCreateFailed(#[from] ShmemError),
}
