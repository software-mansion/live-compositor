use std::path::PathBuf;
use std::sync::Arc;

use compositor_chromium::cef;
use compositor_render::{EMBED_SOURCE_FRAMES_MESSAGE, UNEMBED_SOURCE_FRAMES_MESSAGE};
use log::error;
use shared_memory::ShmemConf;

use crate::state::{FrameInfo, Source, State};

pub struct RenderProcessHandler {
    state: Arc<State>,
}

impl cef::RenderProcessHandler for RenderProcessHandler {
    fn on_process_message_received(
        &mut self,
        _browser: &cef::Browser,
        frame: &cef::Frame,
        _source_process: cef::ProcessId,
        message: &cef::ProcessMessage,
    ) -> bool {
        match message.name().as_str() {
            EMBED_SOURCE_FRAMES_MESSAGE => self.handle_embed_sources(message, frame),
            UNEMBED_SOURCE_FRAMES_MESSAGE => self.handle_unembed_source(message, frame),
            name => error!("Unknown message type: {name}"),
        }
        false
    }
}

impl RenderProcessHandler {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }

    fn handle_embed_sources(&self, msg: &cef::ProcessMessage, surface: &cef::Frame) {
        let ctx = surface.v8_context().unwrap();
        let ctx_entered = ctx.enter().unwrap();
        let cef::V8Value::Object(mut global) = ctx.global().unwrap() else {
            panic!("Expected global to be an object");
        };

        const MSG_SIZE: usize = 3;
        for i in (0..msg.size()).step_by(3) {
            let source_idx = i / MSG_SIZE;

            let Some(shmem_path) = msg.read_string(i) else {
                error!("Failed to read shared memory path at {i}");
                continue;
            };
            let shmem_path = PathBuf::from(shmem_path);

            let Some(width) = msg.read_int(i + 1) else {
                error!("Failed to read width of input {} at {}", source_idx, i + 1);
                continue;
            };

            let Some(height) = msg.read_int(i + 2) else {
                error!("Failed to read height of input {} at {}", source_idx, i + 2);
                continue;
            };

            if !self.state.contains_source(&shmem_path) {
                let frame_info = FrameInfo {
                    source_idx,
                    width: width as u32,
                    height: height as u32,
                    shmem_path,
                };

                self.embed_frame(frame_info, &mut global, &ctx_entered);
            }
        }
    }

    fn embed_frame(
        &self,
        frame_info: FrameInfo,
        global: &mut cef::V8Object,
        ctx_entered: &cef::V8ContextEntered,
    ) {
        let shmem = ShmemConf::new()
            .flink(&frame_info.shmem_path)
            .open()
            .unwrap();
        let data_ptr = shmem.as_ptr();

        let array_buffer: cef::V8Value = unsafe {
            cef::V8ArrayBuffer::from_ptr(
                data_ptr,
                (4 * frame_info.width * frame_info.height) as usize,
                ctx_entered,
            )
        }
        .into();

        // TODO: Figure out emedding API
        // NOTE TO REVIEWERS: The section below is not part of this PR
        // Currently we pass frame data, width and height to JS context.
        // User has to handle this data manually. This approach is not really ergonomic and elegant
        global
            .set(
                &format!("input_{}_data", frame_info.source_idx),
                &array_buffer,
                cef::V8PropertyAttribute::DoNotDelete,
                ctx_entered,
            )
            .unwrap();

        global
            .set(
                &format!("input_{}_width", frame_info.source_idx),
                &cef::V8Uint::new(frame_info.width).into(),
                cef::V8PropertyAttribute::DoNotDelete,
                ctx_entered,
            )
            .unwrap();

        global
            .set(
                &format!("input_{}_height", frame_info.source_idx),
                &cef::V8Uint::new(frame_info.height).into(),
                cef::V8PropertyAttribute::DoNotDelete,
                ctx_entered,
            )
            .unwrap();
        // ------

        self.state.insert_source(
            frame_info.shmem_path.clone(),
            Source {
                _shmem: shmem,
                _array_buffer: array_buffer,
                info: frame_info,
            },
        );
    }

    fn handle_unembed_source(&self, msg: &cef::ProcessMessage, surface: &cef::Frame) {
        let Some(shmem_path) = msg.read_string(0) else {
            error!("Failed to read node ID");
            return;
        };
        let shmem_path = PathBuf::from(shmem_path);
        let Some(source_idx) = self.state.source_index(&shmem_path) else {
            error!("Source {shmem_path:?} not found");
            return;
        };
        let ctx = surface.v8_context().unwrap();
        let ctx_entered = ctx.enter().unwrap();

        // NOTE: This will change once embedding API is finished
        let source_id = format!("input_{}_data", source_idx);
        if let cef::V8Value::Object(mut global) = ctx.global().unwrap() {
            global.delete(&source_id, &ctx_entered).unwrap();
        } else {
            panic!("Expected global to be an object");
        }

        self.state.remove_source(&shmem_path);
    }
}
