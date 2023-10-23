use anyhow::Result;
use compositor_common::{
    scene::{InputId, NodeId, OutputId, Resolution, SceneSpec},
    Frame,
};
use compositor_render::{FrameSet, Renderer};
use std::{collections::HashSet, fmt::Display, fs, path::PathBuf, sync::Arc, time::Duration};

use super::{
    test_case::TestInput,
    utils::{are_snapshots_equal, frame_to_rgba, SNAPSHOTS_DIR_NAME},
};

pub struct SceneTest {
    pub test_name: &'static str,
    pub scene: Arc<SceneSpec>,
    pub inputs: Vec<TestInput>,
    pub renderer: Renderer,
    pub timestamps: Vec<Duration>,
    pub outputs: Vec<&'static str>,
}

impl SceneTest {
    pub fn run(&self) -> Result<()> {
        let mut produced_outputs = HashSet::new();
        let snapshots = self.generate_snapshots()?;
        for new_snapshot in snapshots.iter() {
            produced_outputs.insert(new_snapshot.output_id.to_string());

            let save_path = new_snapshot.save_path();
            if !save_path.exists() {
                return Err(SceneTestError::SnapshotNotFound(new_snapshot.clone()).into());
            }

            let old_snapshot_data = image::open(&save_path)?.to_rgba8();
            if !are_snapshots_equal(&old_snapshot_data, &new_snapshot.data) {
                return Err(SceneTestError::Mismatch(new_snapshot.clone()).into());
            }
        }

        // Check if every output was produced
        for output_id in self.outputs.iter() {
            let was_present = produced_outputs.remove(*output_id);
            if !was_present {
                return Err(SceneTestError::OutputsNotFound(output_id).into());
            }
        }

        self.check_for_unused_snapshots(&snapshots)?;
        Ok(())
    }

    pub fn generate_snapshots(&self) -> Result<Vec<Snapshot>> {
        let snapshots = self
            .timestamps
            .iter()
            .cloned()
            .map(|pts| self.snapshots_for_pts(pts))
            .collect::<Result<Vec<_>>>()?;

        Ok(snapshots.into_iter().flatten().collect())
    }

    fn snapshots_for_pts(&self, pts: Duration) -> Result<Vec<Snapshot>> {
        let mut frame_set = FrameSet::new(pts);
        for input in self.inputs.iter() {
            let input_id = InputId(NodeId(input.name.into()));
            let frame = Frame {
                data: input.data.clone(),
                resolution: input.resolution,
                pts,
            };
            frame_set.frames.insert(input_id, frame);
        }

        let outputs = self.renderer.render(frame_set)?;
        let output_specs = &self.scene.outputs;
        let mut snapshots = Vec::new();

        for spec in output_specs {
            let output_frame = outputs.frames.get(&spec.output_id).unwrap();
            let new_snapshot = frame_to_rgba(output_frame);
            snapshots.push(Snapshot {
                test_name: self.test_name.to_owned(),
                output_id: spec.output_id.clone(),
                pts,
                resolution: output_frame.resolution,
                data: new_snapshot,
            });
        }

        Ok(snapshots)
    }

    fn check_for_unused_snapshots(&self, snapshots: &[Snapshot]) -> Result<()> {
        let snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(SNAPSHOTS_DIR_NAME)
            .join(self.test_name);
        let parent_path = snapshot_path.parent().unwrap();

        for entry in fs::read_dir(parent_path).unwrap() {
            let entry = entry.unwrap();
            if !entry.file_name().to_string_lossy().ends_with(".bmp") {
                continue;
            }
            if !snapshots.iter().any(|s| s.save_path() == entry.path()) {
                return Err(SceneTestError::UnusedSnapshot(entry.path()).into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub test_name: String,
    pub output_id: OutputId,
    pub pts: Duration,
    pub resolution: Resolution,
    pub data: Vec<u8>,
}

impl Snapshot {
    pub fn save_path(&self) -> PathBuf {
        let snapshots_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SNAPSHOTS_DIR_NAME);

        let out_file_name = format!(
            "{}_{}_{}.bmp",
            self.test_name,
            self.pts.as_millis(),
            self.output_id
        );
        snapshots_path.join(out_file_name)
    }
}

#[derive(Debug)]
pub enum SceneTestError {
    SnapshotNotFound(Snapshot),
    Mismatch(Snapshot),
    UnusedSnapshot(PathBuf),
    OutputsNotFound(&'static str),
}

impl std::error::Error for SceneTestError {}

impl Display for SceneTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_msg = match self {
            SceneTestError::SnapshotNotFound(Snapshot {
                test_name,
                output_id,
                pts,
                ..
            }) => format!(
                "Test \"{}\": OutputId({}) & PTS({}). Snapshot file not found. Generate snapshots first",
                test_name,
                output_id,
                pts.as_secs()
            ),
            SceneTestError::Mismatch(Snapshot {
                test_name,
                output_id,
                pts,
                ..
            }) => {
                format!(
                    "Test \"{}\": OutputId({}) & PTS({}). Snapshots are different",
                    test_name,
                    output_id,
                    pts.as_secs_f32()
                )
            }
            SceneTestError::UnusedSnapshot(path) => format!("Snapshot \"{}\" was not used during testing", path.to_string_lossy()),
            SceneTestError::OutputsNotFound(output_id) => format!("Output \"{output_id}\" is missing"),
        };

        f.write_str(&err_msg)
    }
}
