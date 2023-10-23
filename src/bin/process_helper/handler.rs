use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use compositor_chromium::cef;
use compositor_render::{
    EMBED_SOURCE_FRAMES_MESSAGE, GET_FRAME_POSITIONS_MESSAGE, UNEMBED_SOURCE_FRAMES_MESSAGE,
};
use log::{debug, error};

use crate::state::{FrameInfo, State};

pub struct RenderProcessHandler {
    state: Arc<Mutex<State>>,
}

impl cef::RenderProcessHandler for RenderProcessHandler {
    fn on_context_created(
        &mut self,
        _browser: &cef::Browser,
        _frame: &cef::Frame,
        context: &cef::V8Context,
    ) {
        let mut global = context.global().unwrap();
        let ctx_entered = context.enter().unwrap();

        context.eval(include_str!("render_frame.js")).unwrap();
        if let Err(err) = self.register_native_funcs(&mut global, &ctx_entered) {
            error!("Failed to register native functions for V8Context: {err}");
        }
    }

    fn on_process_message_received(
        &mut self,
        _browser: &cef::Browser,
        frame: &cef::Frame,
        _source_process: cef::ProcessId,
        message: &cef::ProcessMessage,
    ) -> bool {
        let result = match message.name().as_str() {
            EMBED_SOURCE_FRAMES_MESSAGE => self.embed_sources(message, frame),
            UNEMBED_SOURCE_FRAMES_MESSAGE => self.unembed_source(message, frame),
            GET_FRAME_POSITIONS_MESSAGE => self.send_frame_positions(message, frame),
            name => Err(anyhow!("Unknown message type: {name}")),
        };

        if let Err(err) = result {
            error!("Error occurred while processing IPC message: {err}");
            // Message was not handled
            return false;
        }

        // Message was handled
        true
    }
}

impl RenderProcessHandler {
    pub fn new(state: Arc<Mutex<State>>) -> Self {
        Self { state }
    }

    fn embed_sources(&self, msg: &cef::ProcessMessage, surface: &cef::Frame) -> Result<()> {
        let ctx = surface.v8_context()?;
        let ctx_entered = ctx.enter()?;
        let mut global = ctx.global()?;

        const MSG_SIZE: usize = 3;
        for i in (0..msg.size()).step_by(3) {
            let source_idx = i / MSG_SIZE;

            let Some(shmem_path) = msg.read_string(i) else {
                return Err(anyhow!("Failed to read shared memory path at {i}"));
            };
            let shmem_path = PathBuf::from(shmem_path);

            let Some(width) = msg.read_int(i + 1) else {
                return Err(anyhow!(
                    "Failed to read width of input {} at {}",
                    source_idx,
                    i + 1
                ));
            };

            let Some(height) = msg.read_int(i + 2) else {
                return Err(anyhow!(
                    "Failed to read height of input {} at {}",
                    source_idx,
                    i + 2
                ));
            };

            if width == 0 && height == 0 {
                continue;
            }

            let frame_info = FrameInfo {
                source_idx,
                width: width as u32,
                height: height as u32,
                shmem_path,
            };

            self.render_frame(frame_info, &mut global, &ctx_entered)?;
        }

        Ok(())
    }

    fn render_frame(
        &self,
        frame_info: FrameInfo,
        global: &mut cef::V8Global,
        ctx_entered: &cef::V8ContextEntered,
    ) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        let source = match state.source(&frame_info.shmem_path) {
            Some(source) => source,
            None => state.create_source(frame_info, ctx_entered)?,
        };

        global.call_method(
            "renderFrame",
            &[
                &source.source_id,
                &source.array_buffer,
                &source.width,
                &source.height,
            ],
            ctx_entered,
        )?;

        Ok(())
    }

    fn unembed_source(&self, msg: &cef::ProcessMessage, surface: &cef::Frame) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        let Some(shmem_path) = msg.read_string(0) else {
            return Err(anyhow!("Failed to read node ID"));
        };
        let shmem_path = PathBuf::from(shmem_path);
        let Some(source) = state.source(&shmem_path) else {
            debug!("Source {shmem_path:?} not found");
            return Ok(());
        };

        let ctx = surface.v8_context()?;
        let ctx_entered = ctx.enter()?;

        let source_id = state.input_name(source.source_index)?;
        let mut global = ctx.global()?;
        global.delete(&source_id, &ctx_entered)?;
        state.remove_source(&shmem_path);

        Ok(())
    }

    fn send_frame_positions(&self, msg: &cef::ProcessMessage, surface: &cef::Frame) -> Result<()> {
        let ctx = surface.v8_context()?;
        let ctx_entered = ctx.enter()?;
        let global = ctx.global()?;
        let document = global.document()?;

        let Some(source_count) = msg.read_int(0) else {
            return Err(anyhow!("Expected source count"));
        };

        let state = self.state.lock().unwrap();
        let mut response = cef::ProcessMessage::new(GET_FRAME_POSITIONS_MESSAGE);
        let mut index = 0;
        for source_idx in 0..(source_count as usize) {
            let source_id = state.input_name(source_idx)?;
            let element = match document.element_by_id(&source_id, &ctx_entered) {
                Ok(element) => element,
                Err(err) => {
                    return Err(anyhow!("Failed to retrieve element \"{source_id}\": {err}"));
                }
            };
            let rect = element.bounding_rect(&ctx_entered)?;

            response.write_double(index, rect.x);
            response.write_double(index + 1, rect.y);
            response.write_double(index + 2, rect.width);
            response.write_double(index + 3, rect.height);

            index += 4;
        }

        surface.send_process_message(cef::ProcessId::Browser, response)?;

        Ok(())
    }

    pub fn register_native_funcs(
        &self,
        global: &mut cef::V8Global,
        ctx_entered: &cef::V8ContextEntered,
    ) -> Result<()> {
        let state = self.state.clone();
        let func = cef::V8Function::new("register_inputs", move |args| {
            let mut input_mappings = Vec::new();
            for arg in args {
                let cef::V8Value::String(element_id) = arg else {
                    return Err("Expected string value".into());
                };
                let element_id = element_id.get().unwrap().into();
                if input_mappings.contains(&element_id) {
                    return Err(format!(
                        "\"{element_id}\" already exists in the provided input mappings"
                    )
                    .into());
                }

                input_mappings.push(element_id);
            }

            let mut state = state.lock().unwrap();
            state.set_input_mappings(input_mappings);
            Ok(cef::V8Undefined::new().into())
        });

        global.set(
            "register_inputs",
            &cef::V8Value::from(func),
            cef::V8PropertyAttribute::None,
            ctx_entered,
        )?;

        Ok(())
    }
}
