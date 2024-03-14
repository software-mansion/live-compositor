use std::env;

use compositor_integration_tests::integration_tests;
use compositor_render::use_global_wgpu_ctx;
use libtest_mimic::{Arguments, Trial};
use video_compositor::logger;

pub fn main() {
    env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "0");
    ffmpeg_next::format::network::init();
    logger::init_logger();

    use_global_wgpu_ctx();

    let args = Arguments::from_args();
    let tests = integration_tests()
        .into_iter()
        .map(|test| Trial::test(test.name, move || test.run_test().map_err(Into::into)))
        .collect();

    libtest_mimic::run(&args, tests).exit();
}
