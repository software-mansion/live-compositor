use compositor_pipeline::{
    pipeline::{GraphicsContext, Options},
    Pipeline,
};
use live_compositor::config::read_config;

// This example illustrates how to initialize a GraphicsContext separately to get access to a wgpu
// instance, adapter, queue and device.

#[cfg(target_os = "linux")]
fn main() {
    let graphics_context =
        GraphicsContext::new(false, wgpu::Features::default(), wgpu::Limits::default()).unwrap();

    let _device = graphics_context.device.clone();
    let _queue = graphics_context.queue.clone();

    let _adapter = graphics_context
        .vulkan_ctx
        .as_ref()
        .unwrap()
        .wgpu_ctx
        .adapter
        .clone();

    let _instance = graphics_context
        .vulkan_ctx
        .as_ref()
        .unwrap()
        .wgpu_ctx
        .instance
        .clone();

    let config = read_config();

    let _pipeline = Pipeline::new(Options {
        wgpu_ctx: Some(graphics_context),
        queue_options: config.queue_options,
        stream_fallback_timeout: config.stream_fallback_timeout,
        web_renderer: config.web_renderer,
        force_gpu: config.force_gpu,
        download_root: config.download_root,
        output_sample_rate: config.output_sample_rate,
        wgpu_features: config.required_wgpu_features,
        load_system_fonts: Some(true),
    })
    .unwrap();
}

#[cfg(target_os = "macos")]
fn main() {}
