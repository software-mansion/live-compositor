use std::{env, sync::Arc};

use compositor_chromium::cef;
use compositor_render::{EMBED_SOURCE_FRAMES_MESSAGE, UNEMBED_SOURCE_FRAMES_MESSAGE};
use log::error;
use shared_memory::ShmemConf;

use crate::state::State;

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

        for i in (0..msg.size()).step_by(3) {
            let Some(source_id) = msg.read_string(i) else {
                error!("Failed to read source ID at {i}");
                continue;
            };

            let Some(width) = msg.read_int(i + 1) else {
                error!("Failed to read width of {} at {}", source_id, i + 1);
                continue;
            };

            let Some(height) = msg.read_int(i + 2) else {
                error!("Failed to read height of {} at {}", source_id, i + 2);
                continue;
            };

            if !self.state.contains_source(&source_id) {
                self.embed_frame(source_id.clone(), width, height, &mut global, &ctx_entered);
            }
        }
    }

    fn embed_frame(
        &self,
        source_id: String,
        width: i32,
        height: i32,
        global: &mut cef::V8Object,
        ctx_entered: &cef::V8ContextEntered,
    ) {
        let shmem = ShmemConf::new()
            .flink(env::temp_dir().join(&source_id))
            .open()
            .unwrap();
        let data_ptr = shmem.as_ptr();
        let array_buffer: cef::V8Value = unsafe {
            cef::V8ArrayBuffer::from_ptr(data_ptr, (4 * width * height) as usize, ctx_entered)
        }
        .into();

        // TODO: Figure out emedding API
        // NOTE TO REVIEWERS: The section below is not part of this PR
        // Currently we pass frame data, width and height to JS context.
        // User has to handle this data manually. This approach is not really ergonomic and elegant
        global
            .set(
                &format!("{source_id}_data"),
                &array_buffer,
                cef::V8PropertyAttribute::DoNotDelete,
                ctx_entered,
            )
            .unwrap();

        global
            .set(
                &format!("{source_id}_width"),
                &cef::V8Int::new(width).into(),
                cef::V8PropertyAttribute::DoNotDelete,
                ctx_entered,
            )
            .unwrap();

        global
            .set(
                &format!("{source_id}_height"),
                &cef::V8Int::new(height).into(),
                cef::V8PropertyAttribute::DoNotDelete,
                ctx_entered,
            )
            .unwrap();
        // ------

        self.state.insert_source(source_id, shmem, array_buffer);
    }

    fn handle_unembed_source(&self, msg: &cef::ProcessMessage, surface: &cef::Frame) {
        let Some(source_id) = msg.read_string(0) else {
            error!("Failed to read source ID");
            return;
        };

        let ctx = surface.v8_context().unwrap();
        let ctx_entered = ctx.enter().unwrap();
        if let cef::V8Value::Object(mut global) = ctx.global().unwrap() {
            global.delete(&source_id, &ctx_entered).unwrap();
        } else {
            panic!("Expected global to be an object");
        }

        self.state.remove_source(&source_id);
    }
}
