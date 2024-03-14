use std::env;

use compositor_integration_tests::integration_tests;
use compositor_render::use_global_wgpu_ctx;
use video_compositor::logger;

fn main() {
    env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "0");
    env::set_var("LIVE_COMPOSITOR_LOGGER_LEVEL", "warn");

    ffmpeg_next::format::network::init();
    logger::init_logger();

    use_global_wgpu_ctx();

    for test in integration_tests() {
        println!("Updating snapshots for test: {}", test.name);
        test.run_update().unwrap();
    }
}
