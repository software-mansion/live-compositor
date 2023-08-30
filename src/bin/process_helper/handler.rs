use std::{path::PathBuf, sync::Arc};

use compositor_chromium::cef;
use compositor_render::{
    EMBED_SOURCE_FRAMES_MESSAGE, SHMEM_FOLDER_PATH, UNEMBED_SOURCE_FRAMES_MESSAGE,
};
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
        let mut global_object = ctx.global().unwrap();

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

            self.embed_frame(
                source_id.clone(),
                width,
                height,
                &mut global_object,
                &ctx_entered,
            );
        }
    }

    fn embed_frame(
        &self,
        source_id: String,
        width: i32,
        height: i32,
        global_object: &mut cef::V8Value,
        ctx_entered: &cef::V8ContextEntered,
    ) {
        if self.state.contains_source(&source_id) {
            return;
        }

        let shmem = ShmemConf::new()
            .flink(PathBuf::from(SHMEM_FOLDER_PATH).join(&source_id))
            .open()
            .unwrap();
        let data_ptr = shmem.as_ptr();
        let array_buffer = unsafe {
            cef::V8Value::array_buffer_from_ptr(
                ctx_entered,
                data_ptr,
                (4 * width * height) as usize,
            )
        };

        // TODO: Figure out emedding API
        // NOTE TO REVIEWERS: The section below is not part of this PR
        // Currently we pass frame data, width and height to JS context.
        // User has to handle this data manually. This approach is not really ergonomic and elegant
        global_object
            .set_value_by_key(
                &format!("{source_id}_data"),
                &array_buffer,
                cef::V8PropertyAttribute::DoNotDelete,
            )
            .unwrap();

        global_object
            .set_value_by_key(
                &format!("{source_id}_width"),
                &cef::V8Value::new_i32(width),
                cef::V8PropertyAttribute::DoNotDelete,
            )
            .unwrap();

        global_object
            .set_value_by_key(
                &format!("{source_id}_height"),
                &cef::V8Value::new_i32(height),
                cef::V8PropertyAttribute::DoNotDelete,
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
        let mut global_object = ctx.global().unwrap();
        global_object
            .delete_value_by_key(&ctx_entered, &source_id)
            .unwrap();

        self.state.remove_source(&source_id);
    }
}
