use core::panic;
use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    sync::{Arc, OnceLock},
    time::Duration,
};

use compositor_render::{
    create_wgpu_ctx, web_renderer, Frame, FrameData, Framerate, Renderer, RendererOptions,
    WgpuFeatures, YuvPlanes,
};

use super::test_case::OUTPUT_ID;

pub const SNAPSHOTS_DIR_NAME: &str = "snapshot_tests/snapshots/render_snapshots";

fn global_wgpu_ctx(
    force_gpu: bool,
    features: wgpu::Features,
) -> (Arc<wgpu::Device>, Arc<wgpu::Queue>) {
    static CTX: OnceLock<(Arc<wgpu::Device>, Arc<wgpu::Queue>)> = OnceLock::new();

    CTX.get_or_init(|| create_wgpu_ctx(force_gpu, features, Default::default()).unwrap())
        .clone()
}

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

pub(super) fn snapshots_diff(old_snapshot: &[u8], new_snapshot: &[u8]) -> f32 {
    if old_snapshot.len() != new_snapshot.len() {
        return 10000.0;
    }
    let square_error: f32 = old_snapshot
        .iter()
        .zip(new_snapshot)
        .map(|(a, b)| (*a as i32 - *b as i32).pow(2) as f32)
        .sum();

    square_error / old_snapshot.len() as f32
}

pub(super) fn create_renderer() -> Renderer {
    let (renderer, _event_loop) = Renderer::new(RendererOptions {
        web_renderer: web_renderer::WebRendererInitOptions {
            enable: false,
            enable_gpu: false,
        },
        force_gpu: false,
        framerate: Framerate { num: 30, den: 1 },
        stream_fallback_timeout: Duration::from_secs(3),
        wgpu_features: WgpuFeatures::default(),
        wgpu_ctx: Some(global_wgpu_ctx(false, Default::default())),
        load_system_fonts: true,
    })
    .unwrap();
    renderer
}

pub fn snapshots_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SNAPSHOTS_DIR_NAME)
}

pub fn find_unused_snapshots(
    produced_snapshots: &HashSet<PathBuf>,
    snapshots_path: PathBuf,
) -> Vec<PathBuf> {
    let mut unused_snapshots = Vec::new();
    for entry in fs::read_dir(snapshots_path).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            let mut snapshots = find_unused_snapshots(produced_snapshots, entry.path());
            unused_snapshots.append(&mut snapshots);
            continue;
        }
        if !entry.file_name().to_string_lossy().ends_with(".png") {
            continue;
        }

        if !produced_snapshots.contains(&entry.path()) {
            unused_snapshots.push(entry.path())
        }
    }

    unused_snapshots
}

pub(super) fn snaphot_save_path(test_name: &str, pts: &Duration) -> PathBuf {
    let out_file_name = format!("{}_{}_{}.png", test_name, pts.as_millis(), OUTPUT_ID);
    snapshots_path().join(out_file_name)
}
