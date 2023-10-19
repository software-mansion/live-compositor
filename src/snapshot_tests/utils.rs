use std::{fmt::Display, fs, path::PathBuf, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use compositor_common::{
    renderer_spec::RendererSpec,
    scene::{OutputId, SceneSpec},
    Frame, Framerate,
};
use compositor_render::{renderer::RendererOptions, FrameSet, Renderer, WebRendererOptions};
use video_compositor::types::{RegisterRequest, Scene};

use super::SNAPSHOTS_DIR_NAME;

pub struct SceneTest {
    pub test_name: String,
    scene: Arc<SceneSpec>,
    renderer: Renderer,
    timestamps: Vec<Duration>,
}

impl SceneTest {
    pub fn new(
        test_name: &str,
        renderers: Vec<RendererSpec>,
        scene: Arc<SceneSpec>,
        timestamps: Vec<Duration>,
    ) -> Result<Self> {
        let (mut renderer, _event_loop) = Renderer::new(RendererOptions {
            web_renderer: WebRendererOptions {
                init: false,
                disable_gpu: false,
            },
            framerate: Framerate(30),
            stream_fallback_timeout: Duration::from_secs(3),
        })?;

        for spec in renderers {
            if matches!(spec, RendererSpec::WebRenderer(_)) {
                return Err(anyhow!("Tests with web renderer are not supported"));
            }
            renderer.register_renderer(spec)?;
        }
        renderer.update_scene(scene.clone())?;

        Ok(Self {
            scene,
            renderer,
            test_name: test_name.to_owned(),
            timestamps,
        })
    }

    pub fn run(&self) -> Result<()> {
        for new_snapshot in self.generate_snapshots()? {
            let save_path = new_snapshot.save_path();
            if !save_path.exists() {
                return Err(SnapshotTestError::NotFound(new_snapshot).into());
            }

            let old_snapshot_data = fs::read(&save_path)?;
            if !are_snapshots_equal(&old_snapshot_data, &new_snapshot.data) {
                return Err(SnapshotTestError::Mismatch(new_snapshot).into());
            }
        }

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
        let frame_set = FrameSet::new(pts);
        let outputs = self.renderer.render(frame_set)?;
        let output_specs = &self.scene.outputs;
        let mut snapshots = Vec::new();

        for spec in output_specs {
            let output_frame = outputs.frames.get(&spec.output_id).unwrap();
            let new_snapshot = frame_to_bytes(output_frame);
            snapshots.push(Snapshot {
                test_name: self.test_name.to_owned(),
                output_id: spec.output_id.clone(),
                pts,
                data: new_snapshot,
            });
        }

        Ok(snapshots)
    }
}

pub fn snapshot_test(
    test_name: &str,
    register_renderer_jsons: Vec<&str>,
    scene_json: &str,
    timestamps: Vec<Duration>,
) -> SceneTest {
    eprintln!("Snapshot test: \"{test_name}\"");

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

    let renderers: Vec<RendererSpec> = register_renderer_jsons
        .into_iter()
        .map(|json| serde_json::from_str(json).unwrap())
        .map(register_requests_to_renderers)
        .collect();

    let scene: Scene = serde_json::from_str(scene_json).unwrap();
    let scene = Arc::new(scene.try_into().unwrap());

    SceneTest::new(test_name, renderers, scene, timestamps).unwrap()
}

fn frame_to_bytes(frame: &Frame) -> Vec<u8> {
    let mut data = Vec::with_capacity(frame.resolution.width * frame.resolution.height * 3 / 2);
    data.extend_from_slice(&frame.data.y_plane);
    data.extend_from_slice(&frame.data.u_plane);
    data.extend_from_slice(&frame.data.v_plane);

    data
}

// TODO: Results may slightly differ depending on the platform. There should be an accepted margin of error here
fn are_snapshots_equal(old_snapshot: &[u8], new_snapshot: &[u8]) -> bool {
    old_snapshot == new_snapshot
}

#[derive(Debug)]
pub struct Snapshot {
    pub test_name: String,
    pub output_id: OutputId,
    pub pts: Duration,
    pub data: Vec<u8>,
}

impl Snapshot {
    pub fn save_path(&self) -> PathBuf {
        let snapshots_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(SNAPSHOTS_DIR_NAME)
            .join("snapshots");
        if !snapshots_path.exists() {
            fs::create_dir_all(&snapshots_path).unwrap();
        }

        let out_file_name = format!(
            "{}_{}_{}.yuv",
            self.test_name,
            self.pts.as_millis(),
            self.output_id
        );
        snapshots_path.join(out_file_name)
    }
}

#[derive(Debug)]
pub enum SnapshotTestError {
    NotFound(Snapshot),
    Mismatch(Snapshot),
}

impl std::error::Error for SnapshotTestError {}

impl Display for SnapshotTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_msg = match self {
            SnapshotTestError::NotFound(Snapshot {
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
            SnapshotTestError::Mismatch(Snapshot {
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
        };

        f.write_str(&err_msg)
    }
}
