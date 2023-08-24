use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};

use compositor_chromium::cef;
use compositor_common::scene::{NodeId, Resolution};
use crossbeam_channel::{bounded, Receiver, Sender};
use log::error;
use shared_memory::{ShmemConf, ShmemError};

use crate::renderer::{
    texture::{utils::pad_to_256, NodeTexture},
    RenderCtx,
};

use super::chromium::ChromiumContext;

pub const EMBED_SOURCE_FRAMES_MESSAGE: &str = "EMBED_SOURCE_FRAMES";
pub const SHMEM_FOLDER_PATH: &str = "shmem";

pub(super) struct BrowserController {
    painted_frames_receiver: Receiver<Vec<u8>>,
    register_embed_sender: Sender<Vec<(NodeId, Resolution)>>,
    shmem_update_sender: Sender<(NodeId, Arc<wgpu::Buffer>, wgpu::Extent3d)>,
    /// Used for synchronizing buffer map and unmap operations
    unmap_signal_receiver: Receiver<()>,
    frame_data: Option<Vec<u8>>,
}

impl BrowserController {
    pub fn new(
        ctx: Arc<ChromiumContext>,
        url: String,
        resolution: Resolution,
    ) -> Result<Self, BrowserControllerNewError> {
        let (painted_frames_sender, painted_frames_receiver) = crossbeam_channel::unbounded();
        let (register_embed_sender, register_embed_receiver) = crossbeam_channel::unbounded();
        let (shmem_update_sender, shmem_update_receiver) = crossbeam_channel::unbounded();
        let (unmap_signal_sender, unmap_signal_receiver) = crossbeam_channel::bounded(0);

        if !Path::new(SHMEM_FOLDER_PATH).exists() {
            std::fs::create_dir_all(SHMEM_FOLDER_PATH)?;
        }

        Self::start_browser(
            ctx,
            url,
            resolution,
            register_embed_receiver,
            painted_frames_sender,
        );
        Self::handle_shmem_updates(shmem_update_receiver, unmap_signal_sender);

        let controller = Self {
            painted_frames_receiver,
            register_embed_sender,
            shmem_update_sender,
            unmap_signal_receiver,
            frame_data: None,
        };

        Ok(controller)
    }

    fn start_browser(
        ctx: Arc<ChromiumContext>,
        url: String,
        resolution: Resolution,
        register_embed_receiver: Receiver<Vec<(NodeId, Resolution)>>,
        painted_frames_sender: Sender<Vec<u8>>,
    ) {
        let client = BrowserClient::new(painted_frames_sender, resolution);

        // Browser does not implement `Send` and `Sync` so we can't share it between threads.
        // We keep it isolated in this thread
        thread::spawn(move || {
            let Ok(browser) = ctx.start_browser(&url, client) else {
                    error!("Couldn't start browser for {url}");
                    return;
                };

            loop {
                let sources_info = register_embed_receiver.recv().unwrap();
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

                let Ok(frame) = browser.main_frame() else {
                        error!("Main frame is no longer available for {url}");
                        return;
                    };

                frame
                    .send_process_message(cef::ProcessId::Renderer, process_message)
                    .expect("send ipc message");
            }
        });
    }

    fn handle_shmem_updates(
        shmem_update_receiver: Receiver<(NodeId, Arc<wgpu::Buffer>, wgpu::Extent3d)>,
        unmap_signal_sender: Sender<()>,
    ) {
        thread::spawn(move || {
            let mut shared_memories = HashMap::new();
            loop {
                let (id, input_buffer, size) = shmem_update_receiver.recv().unwrap();
                let shmem = match shared_memories.get(&id) {
                    Some(shmem) => shmem,
                    None => {
                        let shmem = ShmemConf::new()
                            .flink(PathBuf::from(SHMEM_FOLDER_PATH).join(id.to_string()))
                            .size((4 * size.width * size.height) as usize)
                            .force_create_flink()
                            .create()
                            .expect("create shared memory");

                        shared_memories.insert(id.clone(), shmem);
                        shared_memories.get(&id).unwrap()
                    }
                };

                let shmem = shmem.as_ptr();

                // Writes buffer data to shared memory
                {
                    let range = input_buffer.slice(..).get_mapped_range();
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
                unmap_signal_sender.send(()).unwrap();
            }
        });
    }

    pub fn retrieve_frame(&mut self) -> Option<&[u8]> {
        if let Some(frame) = self.painted_frames_receiver.try_iter().last() {
            self.frame_data.replace(frame);
        }

        self.frame_data.as_deref()
    }

    pub fn send_sources(
        &mut self,
        ctx: &RenderCtx,
        sources: &[(&NodeId, &NodeTexture)],
        buffers: &[Arc<wgpu::Buffer>],
    ) -> Result<(), EmbedFrameError> {
        let mut pending_downloads = Vec::new();
        for ((id, texture), buffer) in sources.iter().zip(buffers) {
            let size = texture.rgba_texture().size();
            pending_downloads.push(self.download_buffer((*id).clone(), size, buffer.clone()));
        }

        ctx.wgpu_ctx.device.poll(wgpu::Maintain::Wait);

        for pending in pending_downloads {
            pending()?;
        }

        let sources_info = sources
            .iter()
            .map(|(id, texture)| ((*id).clone(), dbg!(texture.resolution())))
            .collect();
        self.register_embed_sender.send(sources_info).unwrap();

        Ok(())
    }

    fn download_buffer(
        &self,
        id: NodeId,
        size: wgpu::Extent3d,
        source: Arc<wgpu::Buffer>,
    ) -> impl FnOnce() -> Result<(), EmbedFrameError> + '_ {
        let (s, r) = bounded(1);
        source
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                if let Err(err) = s.send(result) {
                    error!("channel send error: {err}")
                }
            });

        move || {
            r.recv().unwrap()?;
            self.shmem_update_sender
                .send((id, source.clone(), size))
                .unwrap();
            self.unmap_signal_receiver.recv().unwrap();
            source.unmap();

            Ok(())
        }
    }
}

pub(super) struct BrowserClient {
    painted_frames_sender: Sender<Vec<u8>>,
    resolution: Resolution,
}

impl cef::Client for BrowserClient {
    type RenderHandlerType = RenderHandler;

    fn render_handler(&self) -> Option<Self::RenderHandlerType> {
        Some(RenderHandler::new(
            self.painted_frames_sender.clone(),
            self.resolution,
        ))
    }
}

impl BrowserClient {
    pub fn new(painted_frames_sender: Sender<Vec<u8>>, resolution: Resolution) -> Self {
        Self {
            painted_frames_sender,
            resolution,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BrowserControllerNewError {
    #[error("Failed to create shared memory directory")]
    SharedMemoryDirCreate(#[from] io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum EmbedFrameError {
    #[error("Failed to create shared memory")]
    SharedMemoryCreate(#[from] ShmemError),

    #[error("Failed to download source frame")]
    DownloadFrame(#[from] wgpu::BufferAsyncError),

    #[error("Browser is no longer alive")]
    BrowserNotAlive(#[from] cef::BrowserError),

    #[error("Could not send IPC message")]
    MessageNotSent(#[from] cef::FrameError),
}

pub(super) struct RenderHandler {
    painted_frames_sender: Sender<Vec<u8>>,
    resolution: Resolution,
}

impl cef::RenderHandler for RenderHandler {
    fn resolution(&self, _browser: &cef::Browser) -> Resolution {
        self.resolution
    }

    fn on_paint(&self, _browser: &cef::Browser, buffer: &[u8], _resolution: Resolution) {
        self.painted_frames_sender
            .send(buffer.to_vec())
            .expect("send frame");
    }
}

impl RenderHandler {
    pub fn new(painted_frames_sender: Sender<Vec<u8>>, resolution: Resolution) -> Self {
        Self {
            painted_frames_sender,
            resolution,
        }
    }
}
