use std::{path::PathBuf, sync::Arc, time::Duration};

use super::{
    input::TestInput,
    snapshot::Snapshot,
    snapshot_save_path,
    utils::{create_renderer, frame_to_rgba},
};

use anyhow::Result;
use compositor_render::{
    scene::Component, Frame, FrameSet, InputId, OutputFrameFormat, OutputId, Renderer, RendererId,
    RendererSpec, Resolution,
};

pub(super) const OUTPUT_ID: &str = "output_1";

#[derive(Debug, Clone)]
pub(super) struct TestCase {
    pub name: &'static str,
    pub inputs: Vec<TestInput>,
    pub renderers: Vec<(RendererId, RendererSpec)>,
    pub timestamps: Vec<Duration>,
    pub scene_updates: Vec<Component>,
    pub only: bool,
    pub allowed_error: f32,
    pub resolution: Resolution,
    pub output_format: OutputFrameFormat,
}

impl Default for TestCase {
    fn default() -> Self {
        Self {
            name: "",
            inputs: Vec::new(),
            renderers: Vec::new(),
            timestamps: vec![Duration::from_secs(0)],
            scene_updates: vec![],
            only: false,
            allowed_error: 1.0,
            resolution: Resolution {
                width: 640,
                height: 360,
            },
            output_format: OutputFrameFormat::PlanarYuv420Bytes,
        }
    }
}

pub(super) enum TestResult {
    Success,
    Failure,
}

impl TestCase {
    pub(super) fn renderer(&self) -> Renderer {
        let mut renderer = create_renderer();
        for (id, spec) in self.renderers.iter() {
            renderer
                .register_renderer(id.clone(), spec.clone())
                .unwrap();
        }

        for (index, _) in self.inputs.iter().enumerate() {
            renderer.register_input(InputId(format!("input_{}", index + 1).into()))
        }

        for update in &self.scene_updates {
            renderer
                .update_scene(
                    OutputId(OUTPUT_ID.into()),
                    self.resolution,
                    self.output_format,
                    update.clone(),
                )
                .unwrap();
        }

        renderer
    }

    pub(super) fn run(&self) -> TestResult {
        if self.name.is_empty() {
            panic!("Snapshot test name has to be provided");
        }
        let mut renderer = self.renderer();
        let mut result = TestResult::Success;

        for pts in self.timestamps.iter().copied() {
            if let TestResult::Failure = self.test_snapshots_for_pts(&mut renderer, pts) {
                result = TestResult::Failure;
            }
        }
        result
    }

    fn test_snapshots_for_pts(&self, renderer: &mut Renderer, pts: Duration) -> TestResult {
        let snapshot = self.snapshot_for_pts(renderer, pts).unwrap();

        let snapshots_diff = snapshot.diff_with_saved();
        if snapshots_diff > 0.0 {
            println!(
                "Snapshot error in range (allowed: {}, current: {})",
                self.allowed_error, snapshots_diff
            );
        }
        if snapshots_diff > self.allowed_error {
            if cfg!(feature = "update_snapshots") {
                println!("UPDATE: \"{}\" (pts: {}ms)", self.name, pts.as_millis(),);
                snapshot.update_on_disk();
            } else {
                println!("FAILED: \"{}\" (pts: {}ms)", self.name, pts.as_millis(),);
                snapshot.write_as_failed_snapshot();
                return TestResult::Failure;
            }
        }
        TestResult::Success
    }

    pub(super) fn snapshot_paths(&self) -> Vec<PathBuf> {
        self.timestamps
            .iter()
            .map(|pts| snapshot_save_path(self.name, pts))
            .collect()
    }

    pub(super) fn snapshot_for_pts(
        &self,
        renderer: &mut Renderer,
        pts: Duration,
    ) -> Result<Snapshot> {
        let mut frame_set = FrameSet::new(pts);
        for input in self.inputs.iter() {
            let input_id = InputId::from(Arc::from(input.name.clone()));
            let frame = Frame {
                data: input.data.clone(),
                resolution: input.resolution,
                pts,
            };
            frame_set.frames.insert(input_id, frame);
        }

        let outputs = renderer.render(frame_set)?;

        let output_frame = outputs.frames.get(&OutputId(OUTPUT_ID.into())).unwrap();
        let new_snapshot = frame_to_rgba(output_frame);
        Ok(Snapshot {
            test_name: self.name.to_owned(),
            pts,
            resolution: output_frame.resolution,
            data: new_snapshot,
        })
    }
}
