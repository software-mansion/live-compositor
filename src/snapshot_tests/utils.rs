use core::panic;
use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use compositor_render::{
    create_wgpu_ctx, web_renderer, Frame, FrameData, Framerate, Renderer, RendererOptions,
    WgpuFeatures, YuvPlanes,
};

pub const SNAPSHOTS_DIR_NAME: &str = "snapshot_tests/snapshots/render_snapshots";

pub(super) fn frame_to_rgba(frame: &Frame) -> Vec<u8> {
    let FrameData::PlanarYuv420(YuvPlanes {
        y_plane,
        u_plane,
        v_plane,
    }) = &frame.data
    else {
        panic!("Wrong pixel format")
    };

    // Renderer can sometimes produce resolution that is not dividable by 2
    let corrected_width = frame.resolution.width - (frame.resolution.width % 2);
    let corrected_height = frame.resolution.height - (frame.resolution.height % 2);

    let mut rgba_data = Vec::with_capacity(y_plane.len() * 4);
    for (i, y_plane) in y_plane
        .chunks(frame.resolution.width)
        .enumerate()
        .take(corrected_height)
    {
        for (j, y) in y_plane.iter().enumerate().take(corrected_width) {
            let y = (*y) as f32;
            let u = u_plane[(i / 2) * (frame.resolution.width / 2) + (j / 2)] as f32;
            let v = v_plane[(i / 2) * (frame.resolution.width / 2) + (j / 2)] as f32;

            let r = (y + 1.40200 * (v - 128.0)).clamp(0.0, 255.0);
            let g = (y - 0.34414 * (u - 128.0) - 0.71414 * (v - 128.0)).clamp(0.0, 255.0);
            let b = (y + 1.77200 * (u - 128.0)).clamp(0.0, 255.0);
            rgba_data.extend_from_slice(&[r as u8, g as u8, b as u8, 255]);
        }
    }

    rgba_data
}

pub(super) fn create_renderer() -> Renderer {
    static CTX: OnceLock<(Arc<wgpu::Device>, Arc<wgpu::Queue>)> = OnceLock::new();
    let wgpu_ctx =
        CTX.get_or_init(|| create_wgpu_ctx(false, Default::default(), Default::default()).unwrap());

    let (renderer, _event_loop) = Renderer::new(RendererOptions {
        web_renderer: web_renderer::WebRendererInitOptions {
            enable: false,
            enable_gpu: false,
        },
        force_gpu: false,
        framerate: Framerate { num: 30, den: 1 },
        stream_fallback_timeout: Duration::from_secs(3),
        wgpu_features: WgpuFeatures::default(),
        wgpu_ctx: Some(wgpu_ctx.clone()),
        load_system_fonts: false,
    })
    .unwrap();
    renderer
}
