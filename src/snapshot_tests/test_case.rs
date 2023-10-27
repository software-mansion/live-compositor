use std::{
    collections::{hash_map::RandomState, HashSet},
    fmt::Display,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use super::utils::{are_snapshots_near_equal, create_renderer, frame_to_rgba, snapshots_path};

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
    pub fn generate_snapshots(&self) -> Result<Vec<Snapshot>> {
        let (renderer, scene) = self.prepare_renderer_and_scene();
        self.validate_scene(&scene)?;

        let snapshots = self
            .timestamps
            .iter()
            .cloned()
            .map(|pts| self.snapshots_for_pts(&renderer, &scene, pts))
            .collect::<Result<Vec<_>>>()?;

        Ok(snapshots.into_iter().flatten().collect())
    }

    pub fn test_snapshots(&self, snapshots: &[Snapshot]) -> Result<()> {
        let mut produced_outputs = HashSet::new();
        for snapshot in snapshots.iter() {
            produced_outputs.insert(snapshot.output_id.to_string());

            let save_path = snapshot.save_path();
            if !save_path.exists() {
                return Err(TestCaseError::SnapshotNotFound(snapshot.clone()).into());
            }

            let snapshot_from_disk = image::open(&save_path)?.to_rgba8();
            if !are_snapshots_near_equal(&snapshot_from_disk, &snapshot.data) {
                return Err(TestCaseError::Mismatch(snapshot.clone()).into());
            }
        }

        // Check if every output was produced
        for output_id in self.outputs.iter() {
            let was_present = produced_outputs.remove(*output_id);
            if !was_present {
                return Err(TestCaseError::OutputNotFound(output_id).into());
            }
        }
        if !produced_outputs.is_empty() {
            return Err(TestCaseError::UnknownOutputs {
                expected: self.outputs.clone(),
                unknown: Vec::from_iter(produced_outputs),
            }
            .into());
        }

        Ok(())
    }

    fn validate_scene(&self, scene: &SceneSpec) -> Result<()> {
        let inputs: HashSet<NodeId, RandomState> = HashSet::from_iter(
            self.inputs
                .iter()
                .map(|input| NodeId(input.name.as_str().into())),
        );
        let outputs: HashSet<NodeId, RandomState> =
            HashSet::from_iter(self.outputs.iter().map(|output| NodeId((*output).into())));

        scene
            .validate(&inputs.iter().collect(), &outputs.iter().collect())
            .map_err(Into::into)
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

#[derive(Debug, Clone)]
pub struct TestInput {
    pub name: String,
    pub resolution: Resolution,
    pub data: YuvData,
}

impl TestInput {
    const COLOR_VARIANTS: [(u8, u8, u8); 17] = [
        // RED, input_0
        (255, 0, 0),
        // BLUE, input_1
        (0, 0, 255),
        // GREEN, input_2
        (0, 255, 0),
        // YELLOW, input_3
        (255, 255, 0),
        // MAGENTA, input_4
        (255, 0, 255),
        // CYAN, input_5
        (0, 255, 255),
        // ORANGE, input_6
        (255, 165, 0),
        // WHITE, input_7
        (255, 255, 255),
        // GRAY, input_8
        (128, 128, 128),
        // LIGHT_RED, input_9
        (255, 128, 128),
        // LIGHT_BLUE, input_10
        (128, 128, 255),
        // LIGHT_GREEN, input_11
        (128, 255, 128),
        // PINK, input_12
        (255, 192, 203),
        // PURPLE, input_13
        (128, 0, 128),
        // BROWN, input_14
        (165, 42, 42),
        // YELLOW_GREEN, input_15
        (154, 205, 50),
        // LIGHT_YELLOW, input_16
        (255, 255, 224),
    ];

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
        if index >= Self::COLOR_VARIANTS.len() {
            panic!("Reached input amount limit: {}", Self::COLOR_VARIANTS.len())
        }

        let color = Self::COLOR_VARIANTS[index];
        let r = color.0 as f32;
        let g = color.1 as f32;
        let b = color.2 as f32;

        let y = (r * 0.299 + g * 0.587 + b * 0.144).clamp(0.0, 255.0);
        let u = (r * -0.168736 + g * -0.331264 + b * 0.5 + 128.0).clamp(0.0, 255.0);
        let v = (r * 0.5 + g * -0.418688 + b * -0.081312 + 128.0).clamp(0.0, 255.0);
        let data = YuvData {
            y_plane: vec![y as u8; resolution.width * resolution.height].into(),
            u_plane: vec![u as u8; (resolution.width * resolution.height) / 4].into(),
            v_plane: vec![v as u8; (resolution.width * resolution.height) / 4].into(),
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
        let out_file_name = format!(
            "{}_{}_{}.png",
            self.test_name,
            self.pts.as_millis(),
            self.output_id
        );
        snapshots_path().join(out_file_name)
    }
}

#[derive(Debug)]
pub enum TestCaseError {
    SnapshotNotFound(Snapshot),
    Mismatch(Snapshot),
    OutputNotFound(&'static str),
    UnknownOutputs {
        expected: Vec<&'static str>,
        unknown: Vec<String>,
    },
}

impl std::error::Error for TestCaseError {}

impl Display for TestCaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_msg = match self {
            TestCaseError::SnapshotNotFound(Snapshot {
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
            TestCaseError::Mismatch(Snapshot {
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
            TestCaseError::OutputNotFound(output_id) => format!("Output \"{output_id}\" is missing"),
            TestCaseError::UnknownOutputs { expected, unknown } => format!("Unknown outputs: {unknown:?}. Expected: {expected:?}"),
        };

        f.write_str(&err_msg)
    }
}
