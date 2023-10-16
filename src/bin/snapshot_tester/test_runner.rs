use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::{anyhow, Result};
use compositor_common::{
    renderer_spec::RendererSpec,
    scene::{OutputId, OutputSpec, SceneSpec},
    Framerate,
};
use compositor_render::{
    frame_set::FrameSet, renderer::RendererOptions, Renderer, WebRendererOptions,
};
use serde::Deserialize;

use crate::utils::{ask, frame_to_bytes};

pub struct TestRunner<'a> {
    run_mode: RunMode,
    config_path: &'a Path,
    scene: Arc<SceneSpec>,
    renderer: Renderer,
}

impl<'a> TestRunner<'a> {
    pub fn new(run_mode: RunMode, config_path: &'a Path) -> Result<Self> {
        let config: SnapshotTestConfig = serde_json::from_str(&fs::read_to_string(config_path)?)?;
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
            run_mode,
            config_path,
            scene,
            renderer,
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

            if self.run_mode == RunMode::UpdateSnapshots || !save_path.exists() {
                fs::write(&save_path, new_snapshot)?;
                continue;
            }

            let old_snapshot = fs::read(&save_path)?;
            if Self::compare_snapshots(&old_snapshot, &new_snapshot) {
                continue;
            }

            if self.run_mode == RunMode::Interactive {
                let answer = ask(&format!(
                    "\n\"{}\", PTS({}): Snapshots are different.\nDo you want to overwrite the old snapshot?",
                    self.config_path.to_string_lossy(),
                    pts.as_secs_f32()
                ))?;

                match answer {
                    true => {
                        fs::write(&save_path, new_snapshot)?;
                    }
                    false => {
                        println!("Snapshot check skipped");
                    }
                }

                continue;
            }

            return Err(anyhow!("Snapshots are different."));
        }
        Ok(())
    }

    // TODO: Results may slightly differ depending on the platform. There should be an accepted margin of error here
    fn compare_snapshots(old_snapshot: &[u8], new_snapshot: &[u8]) -> bool {
        old_snapshot == new_snapshot
    }

    fn snapshot_save_path(&self, pts: &Duration, output_id: &OutputId) -> PathBuf {
        let snapshots_path = self.config_path.parent().unwrap().join("snapshots");
        if !snapshots_path.exists() {
            fs::create_dir_all(&snapshots_path).unwrap();
        }
        let out_file_name = format!(
            "{}_{}_{}.yuv",
            self.config_path.file_stem().unwrap().to_string_lossy(),
            pts.as_millis(),
            output_id
        );
        snapshots_path.join(out_file_name)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RunMode {
    Interactive,
    NonInteractive,
    UpdateSnapshots,
}

#[derive(Debug, Deserialize)]
struct SnapshotTestConfig {
    renderers: Vec<RendererSpec>,
    scene: Arc<SceneSpec>,
}
