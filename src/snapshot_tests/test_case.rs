use std::{collections::HashSet, fmt::Display, fs, path::PathBuf, sync::Arc, time::Duration};

use super::utils::{are_snapshots_equal, create_renderer, frame_to_rgba, SNAPSHOTS_DIR_NAME};

use anyhow::Result;
use compositor_common::{
    frame::YuvData,
    renderer_spec::RendererSpec,
    scene::{InputId, NodeId, OutputId, Resolution, SceneSpec},
    Frame,
};
use compositor_render::{FrameSet, Renderer};
use video_compositor::types::{RegisterRequest, Scene};

pub struct TestCase {
    pub name: &'static str,
    pub inputs: Vec<TestInput>,
    pub renderers: Vec<&'static str>,
    pub scene_json: &'static str,
    pub timestamps: Vec<Duration>,
    pub outputs: Vec<&'static str>,
}

impl Default for TestCase {
    fn default() -> Self {
        Self {
            name: "",
            inputs: Vec::new(),
            renderers: Vec::new(),
            scene_json: "",
            timestamps: vec![Duration::from_secs(0)],
            outputs: vec!["output_1"],
        }
    }
}

impl TestCase {
    pub fn run(&self) -> Result<()> {
        let mut produced_outputs = HashSet::new();
        let snapshots = self.generate_snapshots()?;
        for snapshot in snapshots.iter() {
            produced_outputs.insert(snapshot.output_id.to_string());

            let save_path = snapshot.save_path();
            if !save_path.exists() {
                return Err(SceneTestError::SnapshotNotFound(snapshot.clone()).into());
            }

            let snapshot_from_disk = image::open(&save_path)?.to_rgba8();
            if !are_snapshots_equal(&snapshot_from_disk, &snapshot.data) {
                return Err(SceneTestError::Mismatch(snapshot.clone()).into());
            }
        }

        // Check if every output was produced
        for output_id in self.outputs.iter() {
            let was_present = produced_outputs.remove(*output_id);
            if !was_present {
                return Err(SceneTestError::OutputNotFound(output_id).into());
            }
        }
        if !produced_outputs.is_empty() {
            return Err(SceneTestError::UnknownOutputs {
                expected: self.outputs.clone(),
                unknown: Vec::from_iter(produced_outputs),
            }
            .into());
        }

        self.check_for_unused_snapshots(&snapshots)?;
        Ok(())
    }

    pub fn generate_snapshots(&self) -> Result<Vec<Snapshot>> {
        let (renderer, scene) = self.prepare_renderer_and_scene();
        let snapshots = self
            .timestamps
            .iter()
            .cloned()
            .map(|pts| self.snapshots_for_pts(&renderer, &scene, pts))
            .collect::<Result<Vec<_>>>()?;

        Ok(snapshots.into_iter().flatten().collect())
    }

    fn snapshots_for_pts(
        &self,
        renderer: &Renderer,
        scene: &SceneSpec,
        pts: Duration,
    ) -> Result<Vec<Snapshot>> {
        let mut frame_set = FrameSet::new(pts);
        for input in self.inputs.iter() {
            let input_id = InputId(NodeId(input.name.clone().into()));
            let frame = Frame {
                data: input.data.clone(),
                resolution: input.resolution,
                pts,
            };
            frame_set.frames.insert(input_id, frame);
        }

        let outputs = renderer.render(frame_set)?;
        let output_specs = &scene.outputs;
        let mut snapshots = Vec::new();

        for spec in output_specs {
            let output_frame = outputs.frames.get(&spec.output_id).unwrap();
            let new_snapshot = frame_to_rgba(output_frame);
            snapshots.push(Snapshot {
                test_name: self.name.to_owned(),
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
            .join(self.name);
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

    pub fn prepare_renderer_and_scene(&self) -> (Renderer, Arc<SceneSpec>) {
        fn register_requests_to_renderers(register_request: RegisterRequest) -> RendererSpec {
            match register_request {
                RegisterRequest::InputStream(_) | RegisterRequest::OutputStream(_) => {
                    panic!("Input and output streams are not supported in snapshot tests")
                }
                RegisterRequest::Shader(shader) => shader.try_into().unwrap(),
                RegisterRequest::WebRenderer(web_renderer) => web_renderer.try_into().unwrap(),
                RegisterRequest::Image(img) => img.try_into().unwrap(),
            }
        }

        if self.name.is_empty() {
            panic!("Snapshot test name has to be provided");
        }
        let renderers: Vec<RendererSpec> = self
            .renderers
            .iter()
            .cloned()
            .map(|json| serde_json::from_str(json).unwrap())
            .map(register_requests_to_renderers)
            .collect();

        let scene: Scene = serde_json::from_str(self.scene_json).unwrap();
        let scene: Arc<SceneSpec> = Arc::new(scene.try_into().unwrap());

        let renderer = create_renderer(renderers, scene.clone());
        (renderer, scene)
    }
}

#[derive(Debug)]
pub struct TestInput {
    pub name: String,
    pub resolution: Resolution,
    pub data: YuvData,
}

impl TestInput {
    pub fn new(index: usize) -> Self {
        Self::new_with_resolution(
            index,
            Resolution {
                width: 640,
                height: 360,
            },
        )
    }

    pub fn new_with_resolution(index: usize, resolution: Resolution) -> Self {
        let color = ((index * 123) % 255) as u8;

        let data = YuvData {
            y_plane: vec![255; resolution.width * resolution.height].into(),
            u_plane: vec![color; (resolution.width * resolution.height) / 4].into(),
            v_plane: vec![color; (resolution.width * resolution.height) / 4].into(),
        };

        Self {
            name: format!("input_{index}"),
            resolution,
            data,
        }
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
    OutputNotFound(&'static str),
    UnknownOutputs {
        expected: Vec<&'static str>,
        unknown: Vec<String>,
    },
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
            SceneTestError::OutputNotFound(output_id) => format!("Output \"{output_id}\" is missing"),
            SceneTestError::UnknownOutputs { expected, unknown } => format!("Unknown outputs: {unknown:?}. Expected: {expected:?}"),
        };

        f.write_str(&err_msg)
    }
}
