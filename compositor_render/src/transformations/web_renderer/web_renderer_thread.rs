use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use bytes::Bytes;
use compositor_chromium::cef;
use crossbeam_channel::Receiver;
use log::error;

use crate::error::ErrorStack;
use crate::scene::ComponentId;
use crate::state::RegisterCtx;
use crate::transformations::layout::transformation_matrices::Position;
use crate::transformations::web_renderer::chromium_context::ChromiumContext;
use crate::transformations::web_renderer::web_renderer_thread::shared_memory::SharedMemory;
use crate::wgpu::texture::utils::pad_to_256;
use crate::{RendererId, Resolution};

use self::browser_client::BrowserClient;
use self::communication::{
    ResponseSender, UpdateSharedMemoryPayload, WebRendererThreadRequest, DROP_SHARED_MEMORY,
    EMBED_FRAMES_MESSAGE, GET_FRAME_POSITIONS_MESSAGE,
};
use self::shared_memory::SharedMemoryError;

use super::{WebRenderer, WebRendererSpec};

mod browser_client;
pub(super) mod communication;
mod shared_memory;

pub(super) struct WebRendererThread {
    chromium_ctx: Arc<ChromiumContext>,
    spec: WebRendererSpec,

    request_receiver: Receiver<WebRendererThreadRequest>,
    frame_data: Arc<Mutex<Bytes>>,
}

impl WebRendererThread {
    pub fn new(
        ctx: &RegisterCtx,
        spec: WebRendererSpec,
        request_receiver: Receiver<WebRendererThreadRequest>,
    ) -> Self {
        Self {
            chromium_ctx: ctx.chromium.clone(),
            spec,
            request_receiver,
            frame_data: Arc::new(Mutex::new(Bytes::new())),
        }
    }

    pub fn spawn(self) -> JoinHandle<()> {
        thread::Builder::new()
            .name(format!("web renderer thread: {}", self.spec.url))
            .spawn(move || self.run())
            .unwrap()
    }

    fn run(self) {
        let (frame_positions_sender, frame_positions_receiver) = crossbeam_channel::bounded(1);
        let (shared_memory_dropped_sender, shared_memory_dropped_receiver) =
            crossbeam_channel::bounded(1);

        let browser_client = BrowserClient::new(
            self.frame_data.clone(),
            frame_positions_sender,
            shared_memory_dropped_sender,
            self.spec.resolution,
        );
        let Ok(browser) = self
            .chromium_ctx
            .start_browser(&self.spec.url, browser_client)
        else {
            error!("Couldn't start browser for {}", self.spec.url);
            return;
        };

        let mut state = ThreadState::new(
            browser,
            self.chromium_ctx.instance_id(),
            &self.spec.instance_id,
        );
        loop {
            let result = match self.request_receiver.recv().unwrap() {
                WebRendererThreadRequest::GetRenderedWebsite { response_sender } => {
                    self.send_frame_data(response_sender)
                }
                WebRendererThreadRequest::EmbedSources {
                    resolutions,
                    children_ids,
                } => self.embed_sources(&mut state, resolutions, children_ids),
                WebRendererThreadRequest::EnsureSharedMemory { resolutions } => self
                    .ensure_shared_memory(&mut state, &shared_memory_dropped_receiver, resolutions),
                WebRendererThreadRequest::UpdateSharedMemory {
                    payload,
                    response_sender,
                } => self.update_shared_memory(&mut state, payload, response_sender),
                WebRendererThreadRequest::GetFramePositions {
                    children_ids,
                    response_sender,
                } => self.retrieve_frame_positions(
                    &state,
                    children_ids,
                    &frame_positions_receiver,
                    response_sender,
                ),
                WebRendererThreadRequest::Quit => return,
            };

            if let Err(err) = result {
                error!(
                    "Error occurred in chromium sender thread.\n{}",
                    ErrorStack::new(&err).into_string()
                );
            }
        }
    }

    fn send_frame_data(
        &self,
        response_sender: ResponseSender<Option<Bytes>>,
    ) -> Result<(), WebRendererThreadError> {
        let frame_data = self.frame_data.lock().unwrap();
        let frame_data = match frame_data.is_empty() {
            false => Some(frame_data.clone()),
            true => None,
        };

        response_sender
            .send(frame_data)
            .map_err(|_| WebRendererThreadError::ResponseSendFailed)
    }

    fn embed_sources(
        &self,
        state: &mut ThreadState,
        resolutions: Vec<Option<Resolution>>,
        children_ids: Vec<ComponentId>,
    ) -> Result<(), WebRendererThreadError> {
        let mut process_message = cef::ProcessMessageBuilder::new(EMBED_FRAMES_MESSAGE);

        // IPC message to chromium renderer subprocess consists of:
        // - shared memory path
        // - ID attribute of an HTML element
        // - texture width
        // - texture height
        for (source_idx, (resolution, id)) in resolutions.into_iter().zip(children_ids).enumerate()
        {
            let Some(Resolution { width, height }) = resolution else {
                continue;
            };

            process_message.write_string(state.shared_memory(source_idx)?.to_path_string())?;
            process_message.write_string(id.to_string())?;
            process_message.write_int(width as i32)?;
            process_message.write_int(height as i32)?;
        }

        let frame = state.browser.main_frame()?;
        frame.send_process_message(cef::ProcessId::Renderer, process_message.build())?;

        Ok(())
    }

    fn ensure_shared_memory(
        &self,
        state: &mut ThreadState,
        shared_memory_dropped_receiver: &Receiver<()>,
        resolutions: Vec<Option<Resolution>>,
    ) -> Result<(), WebRendererThreadError> {
        fn size_from_resolution(resolution: Option<Resolution>) -> usize {
            match resolution {
                Some(res) => 4 * res.width * res.height,
                None => 1,
            }
        }

        let frame = state.browser.main_frame()?;
        let shared_memory = &mut state.shared_memory;

        // Ensure size for already existing shared memory
        for (resolution, shmem) in resolutions.iter().zip(shared_memory.iter_mut()) {
            let size = size_from_resolution(*resolution);

            // We resize shared memory only when it's too small.
            // This avoids some crashes caused by the lack of resize synchronization.
            if shmem.len() < size {
                let mut process_message = cef::ProcessMessage::new(DROP_SHARED_MEMORY);
                process_message.write_string(0, shmem.to_path_string())?;
                frame.send_process_message(cef::ProcessId::Renderer, process_message)?;

                shared_memory_dropped_receiver.recv_timeout(Duration::from_secs(2))?;
                shmem.resize(size)?;
            }
        }

        // Create additional shared memory
        for (source_idx, resolution) in resolutions
            .into_iter()
            .enumerate()
            .skip(shared_memory.len())
        {
            let size = size_from_resolution(resolution);
            shared_memory.push(SharedMemory::new(
                &state.shared_memory_root_path,
                source_idx,
                size,
            )?);
        }

        Ok(())
    }

    fn update_shared_memory(
        &self,
        state: &mut ThreadState,
        payload: UpdateSharedMemoryPayload,
        response_sender: ResponseSender<()>,
    ) -> Result<(), WebRendererThreadError> {
        let UpdateSharedMemoryPayload {
            source_idx,
            buffer,
            size,
        } = payload;

        // Writes buffer data to shared memory
        let shared_memory = state.shared_memory(source_idx)?;
        {
            let range = buffer.slice(..).get_mapped_range();
            let chunks = range.chunks((4 * pad_to_256(size.width)) as usize);
            for (i, chunk) in chunks.enumerate() {
                let bytes_len = (4 * size.width) as usize;
                shared_memory.write(&chunk[..bytes_len], i * bytes_len)?;
            }
        }

        response_sender
            .send(())
            .map_err(|_| WebRendererThreadError::ResponseSendFailed)
    }

    fn retrieve_frame_positions(
        &self,
        state: &ThreadState,
        children_ids: Vec<ComponentId>,
        frame_positions_receiver: &Receiver<Vec<Position>>,
        response_sender: ResponseSender<Vec<Position>>,
    ) -> Result<(), WebRendererThreadError> {
        let mut message = cef::ProcessMessage::new(GET_FRAME_POSITIONS_MESSAGE);
        for (index, id) in children_ids.into_iter().enumerate() {
            message.write_string(index, id.to_string())?;
        }

        let frame = state.browser.main_frame()?;
        frame.send_process_message(cef::ProcessId::Renderer, message)?;

        let frame_positions = frame_positions_receiver.recv_timeout(Duration::from_secs(2))?;

        response_sender
            .send(frame_positions)
            .map_err(|_| WebRendererThreadError::ResponseSendFailed)
    }
}

struct ThreadState {
    browser: cef::Browser,
    shared_memory: Vec<SharedMemory>,
    shared_memory_root_path: PathBuf,
}

impl Drop for ThreadState {
    fn drop(&mut self) {
        if let Err(err) = self.browser.close() {
            error!("Failed to close browser: {err}")
        }
    }
}

impl ThreadState {
    fn new(browser: cef::Browser, compositor_id: &str, web_renderer_id: &RendererId) -> Self {
        let shared_memory_root_path =
            WebRenderer::shared_memory_root_path(compositor_id, &web_renderer_id.to_string());
        let shared_memory = Vec::new();

        Self {
            browser,
            shared_memory,
            shared_memory_root_path,
        }
    }

    fn shared_memory(
        &mut self,
        source_idx: usize,
    ) -> Result<&mut SharedMemory, WebRendererThreadError> {
        self.shared_memory
            .get_mut(source_idx)
            .ok_or(WebRendererThreadError::SharedMemoryNotAllocated { source_idx })
    }
}

#[derive(Debug, thiserror::Error)]
enum WebRendererThreadError {
    #[error("Browser is no longer alive")]
    BrowserNotAlive(#[from] cef::BrowserError),

    #[error("Browser frame is no longer alive")]
    FrameNotAlive(#[from] cef::FrameError),

    #[error(transparent)]
    SharedMemoryError(#[from] SharedMemoryError),

    #[error("Shared memory should already be allocated for \"{source_idx}\" input")]
    SharedMemoryNotAllocated { source_idx: usize },

    #[error(transparent)]
    ProcessMessageError(#[from] cef::ProcessMessageError),

    #[error("Failed to receive response from browser client")]
    BrowserMessageTimeout(#[from] crossbeam_channel::RecvTimeoutError),

    #[error("Failed to send response")]
    ResponseSendFailed,
}
