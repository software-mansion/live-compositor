use std::{fs, path::PathBuf, sync::Arc, time::Duration};

use anyhow::{anyhow, Result};
use compositor_common::{
    renderer_spec::RendererSpec,
    scene::{OutputId, SceneSpec},
    Frame, Framerate,
};
use compositor_render::{renderer::RendererOptions, FrameSet, Renderer, WebRendererOptions};
use video_compositor::types::{RegisterRequest, Scene};

use super::SNAPSHOTS_DIR_NAME;

struct TestRunner<'a> {
    test_name: &'a str,
    scene: Arc<SceneSpec>,
    renderer: Renderer,
}

impl<'a> TestRunner<'a> {
    pub fn new(
        test_name: &'a str,
        renderers: Vec<RendererSpec>,
        scene: Arc<SceneSpec>,
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
            .join(SNAPSHOTS_DIR_NAME)
            .join("snapshots");
        if !snapshots_path.exists() {
            fs::create_dir_all(&snapshots_path).unwrap();
        }
        let out_file_name = format!("{}_{}_{}.yuv", self.test_name, pts.as_millis(), output_id);
        snapshots_path.join(out_file_name)
    }
}

pub fn test(
    test_name: &str,
    register_renderers_json: &str,
    scene_json: &str,
    timestamps: Vec<Duration>,
) {
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

    let register_renderers_requests: Vec<RegisterRequest> =
        serde_json::from_str(register_renderers_json).unwrap();
    let renderers = register_renderers_requests
        .into_iter()
        .map(register_requests_to_renderers)
        .collect();

    let scene: Scene = serde_json::from_str(scene_json).unwrap();
    let scene = Arc::new(scene.try_into().unwrap());

    let test_runner = TestRunner::new(test_name, renderers, scene).unwrap();
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
