use std::{sync::Arc, time::Duration};

use super::test_case::TestInput;
use compositor_common::{
    frame::YuvData, renderer_spec::RendererSpec, scene::SceneSpec, Frame, Framerate,
};
use compositor_render::{renderer::RendererOptions, Renderer, WebRendererOptions};

pub const SNAPSHOTS_DIR_NAME: &str = "snapshot_tests/snapshots/render_snapshots";

pub(super) fn frame_to_rgba(frame: &Frame) -> Vec<u8> {
    let YuvData {
        y_plane,
        u_plane,
        v_plane,
    } = &frame.data;

    let mut rgba_data = Vec::with_capacity(y_plane.len() * 4);
    for (i, y_plane) in y_plane.chunks(frame.resolution.width).enumerate() {
        for (j, y) in y_plane.iter().enumerate() {
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

// TODO: Results may slightly differ depending on the platform. There should be an accepted margin of error here
pub(super) fn are_snapshots_equal(old_snapshot: &[u8], new_snapshot: &[u8]) -> bool {
    old_snapshot == new_snapshot
}

pub(super) fn populate_test_inputs(inputs: &mut [TestInput]) {
    for (i, input) in inputs.iter_mut().enumerate() {
        let color = ((i * 123) % 255) as u8;
        let width = input.resolution.width;
        let height = input.resolution.height;
        input.data = YuvData {
            y_plane: vec![255; width * height].into(),
            u_plane: vec![color; (width * height) / 4].into(),
            v_plane: vec![color; (width * height) / 4].into(),
        };
    }
}

pub(super) fn create_renderer(renderers: Vec<RendererSpec>, scene: Arc<SceneSpec>) -> Renderer {
    let (mut renderer, _event_loop) = Renderer::new(RendererOptions {
        web_renderer: WebRendererOptions {
            init: false,
            disable_gpu: false,
        },
        framerate: Framerate(30),
        stream_fallback_timeout: Duration::from_secs(3),
    })
    .unwrap();

    for spec in renderers {
        if matches!(spec, RendererSpec::WebRenderer(_)) {
            panic!("Tests with web renderer are not supported");
        }
        renderer.register_renderer(spec).unwrap();
    }
    renderer.update_scene(scene.clone()).unwrap();

    renderer
}
