use std::{collections::HashSet, fmt::Display, path::PathBuf, sync::Arc, time::Duration};

use super::utils::{are_snapshots_near_equal, create_renderer, frame_to_rgba, snaphot_save_path};

use anyhow::Result;
use compositor_common::{
    frame::YuvData,
    renderer_spec::RendererSpec,
    scene::{InputId, OutputId, Resolution},
    Frame,
};
use compositor_pipeline::pipeline;
use compositor_render::{scene::OutputScene, FrameSet, Renderer};
use image::ImageBuffer;
use video_compositor::types::{self, RegisterRequest};

pub struct TestCase {
    pub name: &'static str,
    pub inputs: Vec<TestInput>,
    pub renderers: Vec<&'static str>,
    pub timestamps: Vec<Duration>,
    pub outputs: Vec<(&'static str, Resolution)>,
    pub allowed_error: f32,
}

impl Default for TestCase {
    fn default() -> Self {
        Self {
            name: "",
            inputs: Vec::new(),
            renderers: Vec::new(),
            timestamps: vec![Duration::from_secs(0)],
            outputs: vec![],
            allowed_error: 30.0,
        }
    }
}

pub struct TestCaseInstance {
    pub case: TestCase,
    pub scene: Vec<OutputScene>,
    pub renderer: Renderer,
}

impl TestCaseInstance {
    pub fn new(test_case: TestCase) -> TestCaseInstance {
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

        if test_case.name.is_empty() {
            panic!("Snapshot test name has to be provided");
        }
        let renderers: Vec<RendererSpec> = test_case
            .renderers
            .iter()
            .cloned()
            .map(|json| serde_json::from_str(json).unwrap())
            .map(register_requests_to_renderers)
            .collect();

        let scene: Vec<OutputScene> = test_case
            .outputs
            .iter()
            .map(|output| {
                let scene: types::OutputScene = serde_json::from_str(output.0).unwrap();
                let scene: pipeline::OutputScene = scene.try_into().unwrap();
                OutputScene {
                    output_id: scene.output_id,
                    root: scene.root,
                    resolution: output.1,
                }
            })
            .collect();

        let renderer = create_renderer(renderers, scene.clone());
        TestCaseInstance {
            case: test_case,
            scene,
            renderer,
        }
    }

    #[allow(dead_code)]
    pub fn run(&self) -> Result<(), TestCaseError> {
        for pts in self.case.timestamps.iter() {
            let (_, test_result) = self.test_snapshots_for_pts(*pts);
            test_result?;
        }
        Ok(())
    }

    pub fn test_snapshots_for_pts(
        &self,
        pts: Duration,
    ) -> (Vec<Snapshot>, Result<(), TestCaseError>) {
        let snapshots = self.snapshots_for_pts(pts).unwrap();

        for snapshot in snapshots.iter() {
            let save_path = snapshot.save_path();
            if !save_path.exists() {
                return (
                    snapshots.clone(),
                    Err(TestCaseError::SnapshotNotFound(snapshot.clone())),
                );
            }

            let snapshot_from_disk = image::open(&save_path).unwrap().to_rgba8();
            if !are_snapshots_near_equal(
                &snapshot_from_disk,
                &snapshot.data,
                self.case.allowed_error,
            ) {
                return (
                    snapshots.clone(),
                    Err(TestCaseError::Mismatch {
                        snapshot_from_disk: snapshot_from_disk.into(),
                        produced_snapshot: snapshot.clone(),
                    }),
                );
            }
        }

        // Check if every output was produced
        let produced_outputs: HashSet<OutputId> = snapshots
            .iter()
            .map(|snapshot| snapshot.output_id.clone())
            .collect();
        let expected_outputs: HashSet<OutputId> = self
            .scene
            .iter()
            .map(|output| output.output_id.clone())
            .collect();
        if produced_outputs != expected_outputs {
            return (
                snapshots,
                Err(TestCaseError::OutputMismatch {
                    expected: expected_outputs,
                    received: produced_outputs,
                }),
            );
        }

        (snapshots, Ok(()))
    }

    #[allow(dead_code)]
    pub fn snapshot_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for pts in self.case.timestamps.iter() {
            for output in self.scene.iter() {
                paths.push(snaphot_save_path(
                    self.case.name,
                    pts,
                    output.output_id.clone(),
                ));
            }
        }

        paths
    }

    pub fn snapshots_for_pts(&self, pts: Duration) -> Result<Vec<Snapshot>> {
        let mut frame_set = FrameSet::new(pts);
        for input in self.case.inputs.iter() {
            let input_id = InputId::from(Arc::from(input.name.clone()));
            let frame = Frame {
                data: input.data.clone(),
                resolution: input.resolution,
                pts,
            };
            frame_set.frames.insert(input_id, frame);
        }

        let outputs = self.renderer.render(frame_set)?;
        let mut snapshots = Vec::new();

        for scene in &self.scene {
            let output_frame = outputs.frames.get(&scene.output_id).unwrap();
            let new_snapshot = frame_to_rgba(output_frame);
            snapshots.push(Snapshot {
                test_name: self.case.name.to_owned(),
                output_id: scene.output_id.clone(),
                pts,
                resolution: output_frame.resolution,
                data: new_snapshot,
            });
        }

        Ok(snapshots)
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
        snaphot_save_path(&self.test_name, &self.pts, self.output_id.clone())
    }
}

#[derive(Debug)]
pub enum TestCaseError {
    SnapshotNotFound(Snapshot),
    Mismatch {
        snapshot_from_disk: Box<ImageBuffer<image::Rgba<u8>, Vec<u8>>>,
        produced_snapshot: Snapshot,
    },
    OutputMismatch {
        expected: HashSet<OutputId>,
        received: HashSet<OutputId>,
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
                "FAILED: \"{}\", OutputId({}), PTS({}). Snapshot file not found. Generate snapshots first",
                test_name,
                output_id,
                pts.as_secs()
            ),
            TestCaseError::Mismatch{produced_snapshot: Snapshot {
                test_name,
                output_id,
                pts,
                ..
            },
            ..
            } => {
                format!(
                    "FAILED: \"{}\", OutputId({}), PTS({}). Snapshots are different",
                    test_name,
                    output_id,
                    pts.as_secs_f32()
                )
            }
            TestCaseError::OutputMismatch { expected, received } => format!("Mismatched output\nexpected: {expected:#?}\nreceived: {received:#?}"),
        };

        f.write_str(&err_msg)
    }
}
