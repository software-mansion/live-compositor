use std::{fs, path::PathBuf, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};

use compositor_common::{
    renderer_spec::RendererSpec,
    scene::{OutputId, SceneSpec},
    Frame, Framerate,
};
use compositor_render::{
    frame_set::FrameSet, renderer::RendererOptions, Renderer, WebRendererOptions,
};
use serde::Deserialize;

pub const SNAPSHOTS_DIR_NAME: &str = "snapshot_tests";

pub fn snapshot_tests() {
    test(
        include_str!("../snapshot_tests/basic.json"),
        "basic",
        vec![Duration::from_secs(3)],
    )
}

struct TestRunner<'a> {
    test_name: &'a str,
    scene: Arc<SceneSpec>,
    renderer: Renderer,
}

impl<'a> TestRunner<'a> {
    pub fn new(test_name: &'a str, config: SnapshotTestConfig) -> Result<Self> {
        let scene = config.scene.clone();
        let (mut renderer, _event_loop) = Renderer::new(RendererOptions {
            web_renderer: WebRendererOptions {
                init: false,
                disable_gpu: false,
            },
            framerate: Framerate(30),
            stream_fallback_timeout: Duration::from_secs(3),
        })?;

        for spec in config.renderers {
            if matches!(spec, RendererSpec::WebRenderer(_)) {
                return Err(anyhow!("Tests with web renderer are not supported"));
            }
            renderer.register_renderer(spec)?;
        }
        renderer.update_scene(scene.clone())?;

        Ok(Self {
            scene,
            renderer,
            test_name,
        })
    }

    pub fn run(&self, pts: Duration) -> Result<()> {
        let frame_set = FrameSet::new(pts);
        let outputs = self.renderer.render(frame_set)?;
        let output_specs = &self.scene.outputs;
        for spec in output_specs {
            let output_frame = outputs.frames.get(&spec.output_id).unwrap();
            let new_snapshot = frame_to_bytes(output_frame);
            let save_path = self.snapshot_save_path(&pts, &spec.output_id);
            if !save_path.exists() {
                fs::write(&save_path, new_snapshot)?;
                continue;
            }

            let old_snapshot = fs::read(&save_path)?;
            if are_snapshots_equal(&old_snapshot, &new_snapshot) {
                continue;
            }

            return Err(anyhow!("Snapshots are different."));
        }
        Ok(())
    }

    fn snapshot_save_path(&self, pts: &Duration, output_id: &OutputId) -> PathBuf {
        let snapshots_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(SNAPSHOTS_DIR_NAME)
            .join("snapshots");
        if !snapshots_path.exists() {
            fs::create_dir_all(&snapshots_path).unwrap();
        }
        let out_file_name = format!("{}_{}_{}.yuv", self.test_name, pts.as_millis(), output_id);
        snapshots_path.join(out_file_name)
    }
}

#[derive(Debug, Deserialize)]
pub struct SnapshotTestConfig {
    renderers: Vec<RendererSpec>,
    scene: Arc<SceneSpec>,
}

pub fn test(config_source: &str, test_name: &str, timestamps: Vec<Duration>) {
    let config: SnapshotTestConfig = serde_json::from_str(config_source).unwrap();
    let test_runner = TestRunner::new(test_name, config).unwrap();
    for pts in timestamps {
        if let Err(err) = test_runner.run(pts) {
            panic!(
                "Test \"{}\", PTS({}) failed: {}",
                test_name,
                pts.as_secs_f32(),
                err
            );
        }
    }
}

pub fn frame_to_bytes(frame: &Frame) -> Vec<u8> {
    let mut data = Vec::with_capacity(frame.resolution.width * frame.resolution.height * 3 / 2);
    data.extend_from_slice(&frame.data.y_plane);
    data.extend_from_slice(&frame.data.u_plane);
    data.extend_from_slice(&frame.data.v_plane);

    data
}

// TODO: Results may slightly differ depending on the platform. There should be an accepted margin of error here
pub fn are_snapshots_equal(old_snapshot: &[u8], new_snapshot: &[u8]) -> bool {
    old_snapshot == new_snapshot
}

#[test]
fn run_shanpshot_tests() {
    snapshot_tests();
}
