use std::env;

use anyhow::Result;
use compositor_render::use_global_wgpu_ctx;
use video_compositor::logger;

mod simple;

type TestFn = fn(update_dumps: bool) -> Result<()>;

// All integration tests should be added here
pub fn integration_tests() -> Vec<TestFn> {
    vec![simple::run_simple_test]
}

pub fn integration_test_prerequisites() {
    env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "0");
    ffmpeg_next::format::network::init();
    logger::init_logger();

    use_global_wgpu_ctx();
}
