use compositor_chromium::cef;

struct App;

impl cef::App for App {
    type RenderProcessHandlerType = ();

    fn on_before_command_line_processing(
        &mut self,
        process_type: String,
        command_line: &mut cef::CommandLine,
    ) {
        // Check if main process
        if !process_type.is_empty() {
            return;
        }

        #[cfg(target_os = "macos")]
        command_line.append_switch("use-mock-keychain");
        command_line.append_switch("disable-gpu");
    }
}

fn main() {
    let build_path = &std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("..");

    if cef::bundle_app(&build_path).is_err() {
        panic!("Build process helper first");
    }

    let app = App;
    let settings = cef::Settings {
        windowless_rendering_enabled: true,
        log_severity: cef::LogSeverity::Info,
        ..Default::default()
    };

    let ctx = cef::Context::new(app, settings).unwrap();
    ctx.run_message_loop();
}
