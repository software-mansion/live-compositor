use std::{fs, path::Path, process::Stdio};

use compositor_chromium::cef;
use compositor_common::scene::Resolution;

fn bgra_to_png(
    input_file: impl AsRef<Path>,
    output_file: impl AsRef<Path>,
    resolution: Resolution,
) {
    std::process::Command::new("ffmpeg")
        .arg("-f")
        .arg("rawvideo")
        .arg("-pix_fmt")
        .arg("bgra")
        .arg("-video_size")
        .arg(format!("{}x{}", resolution.width, resolution.height))
        .arg("-i")
        .arg(input_file.as_ref().as_os_str())
        .arg(output_file.as_ref().as_os_str())
        .arg("-y")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn ffmpeg")
        .wait()
        .expect("wait");
}

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
        command_line.append_switch("disable-gpu-shader-disk-cache");
        command_line.append_switch("show-fps-counter");
    }
}

struct Client;

impl cef::Client for Client {
    type RenderHandlerType = RenderHandler;

    fn get_render_handler(&self) -> Option<Self::RenderHandlerType> {
        Some(RenderHandler)
    }
}

struct RenderHandler;

impl cef::RenderHandler for RenderHandler {
    fn get_resolution(&self, _browser: cef::Browser<'_>) -> Resolution {
        Resolution {
            width: 1920,
            height: 1080,
        }
    }

    fn on_paint(&self, browser: cef::Browser<'_>, buffer: &[u8], resolution: Resolution) {
        if !browser.is_loading() {
            fs::write("out.raw", buffer).expect("save image buffer");
            bgra_to_png("out.raw", "out.png", resolution);
            fs::remove_file("./out.raw").expect("remove raw image file");
        }
    }
}

fn main() {
    let build_path = &std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("..");

    if cef::bundle_app(&build_path).is_err() {
        panic!("Build process helper first: cargo build");
    }

    let app = App;
    let settings = cef::Settings {
        windowless_rendering_enabled: true,
        log_severity: cef::LogSeverity::Info,
        ..Default::default()
    };

    let ctx = cef::Context::new(app, settings).expect("create_browser");

    let client = Client;
    let window_info = cef::WindowInfo {
        windowless_rendering_enabled: true,
    };
    let browser_settings = cef::BrowserSettings {
        windowless_frame_rate: 60,
    };
    let _ = ctx.start_browser(
        client,
        window_info,
        browser_settings,
        "https://membrane.stream".to_owned(),
    );

    println!("Starting generating images");
    ctx.run_message_loop();
}
