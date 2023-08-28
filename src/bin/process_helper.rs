use compositor_chromium::cef;
use compositor_render::{
    EMBED_SOURCE_FRAMES_MESSAGE, SHMEM_FOLDER_PATH, UNEMBED_SOURCE_FRAMES_MESSAGE,
};
use log::{error, info};
use shared_memory::{Shmem, ShmemConf};
use std::{cell::RefCell, collections::HashMap, error::Error, path::PathBuf, rc::Rc};

type SourceStateMap = Rc<RefCell<HashMap<String, SourceState>>>;

struct App {
    states: SourceStateMap,
}

impl cef::App for App {
    type RenderProcessHandlerType = RenderProcessHandler;

    fn on_before_command_line_processing(
        &mut self,
        process_type: String,
        _command_line: &mut cef::CommandLine,
    ) {
        info!("Chromium {process_type} subprocess started");
    }

    fn render_process_handler(&self) -> Option<Self::RenderProcessHandlerType> {
        Some(RenderProcessHandler {
            states: self.states.clone(),
        })
    }
}

struct RenderProcessHandler {
    states: SourceStateMap,
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
    fn handle_embed_sources(&self, msg: &cef::ProcessMessage, surface: &cef::Frame) {
        let ctx = surface.v8_context().unwrap();
        let ctx_entered = ctx.enter().unwrap();
        let mut global_object = ctx.global().unwrap();

        for i in (0..msg.size()).step_by(3) {
            let Some(source_id) = msg.read_string(i) else {
                error!("Failed to read source ID at {i}");
                continue;
            };

            let Some(width) = msg.read_int(i+1) else {
                error!("Failed to read width of {} at {}", source_id, i + 1);
                continue;
            };

            let Some(height) = msg.read_int(i+2) else {
                error!("Failed to read height of {} at {}", source_id, i + 2);
                continue;
            };

            if !self.states.borrow().contains_key(&source_id) {
                self.embed_frame(
                    source_id.clone(),
                    width,
                    height,
                    &mut global_object,
                    &ctx_entered,
                );
            }
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
        let shmem = ShmemConf::new()
            .flink(PathBuf::from(SHMEM_FOLDER_PATH).join(&source_id))
            .open()
            .unwrap();
        let data_ptr = shmem.as_ptr();
        let array_buffer =
            unsafe { cef::V8Value::array_buffer_from_ptr(ctx_entered, data_ptr, shmem.len()) };

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

        self.states.borrow_mut().insert(
            source_id,
            SourceState {
                _shmem: shmem,
                _array_buffer: array_buffer,
            },
        );
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

        self.states.borrow_mut().remove(&source_id).unwrap();
    }
}

struct SourceState {
    _shmem: Shmem,
    _array_buffer: cef::V8Value,
}

// Subprocess used by chromium
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let app = App {
        states: Rc::new(RefCell::new(HashMap::new())),
    };
    let context = cef::Context::new_helper()?;
    let exit_code = context.execute_process(app);
    std::process::exit(exit_code);
}
