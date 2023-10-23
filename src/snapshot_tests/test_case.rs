use std::{sync::Arc, time::Duration};

use bytes::Bytes;
use compositor_common::{
    frame::YuvData,
    renderer_spec::RendererSpec,
    scene::{Resolution, SceneSpec},
};
use video_compositor::types::{RegisterRequest, Scene};

use super::{
    scene_test::SceneTest,
    utils::{create_renderer, populate_test_inputs},
};

pub struct TestCase {
    pub name: &'static str,
    pub inputs: Vec<TestInput>,
    pub register_renderer_jsons: Vec<&'static str>,
    pub scene_json: &'static str,
    pub timestamps: Vec<Duration>,
    pub outputs: Vec<&'static str>,
}

impl Default for TestCase {
    fn default() -> Self {
        Self {
            name: "",
            inputs: Vec::new(),
            register_renderer_jsons: Vec::new(),
            scene_json: "",
            timestamps: vec![Duration::from_secs(0)],
            outputs: vec!["output_1"],
        }
    }
}

impl TestCase {
    pub fn into_scene_test(mut self) -> SceneTest {
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

        populate_test_inputs(&mut self.inputs);

        let renderers: Vec<RendererSpec> = self
            .register_renderer_jsons
            .into_iter()
            .map(|json| serde_json::from_str(json).unwrap())
            .map(register_requests_to_renderers)
            .collect();

        let scene: Scene = serde_json::from_str(self.scene_json).unwrap();
        let scene: Arc<SceneSpec> = Arc::new(scene.try_into().unwrap());

        let renderer = create_renderer(renderers, scene.clone());

        SceneTest {
            test_name: self.name,
            scene,
            inputs: self.inputs,
            renderer,
            timestamps: self.timestamps,
            outputs: self.outputs,
        }
    }
}

#[derive(Debug)]
pub struct TestInput {
    pub name: &'static str,
    pub resolution: Resolution,
    pub data: YuvData,
}

impl Default for TestInput {
    fn default() -> Self {
        Self {
            name: "",
            resolution: Resolution {
                width: 640,
                height: 360,
            },
            data: YuvData {
                y_plane: Bytes::new(),
                u_plane: Bytes::new(),
                v_plane: Bytes::new(),
            },
        }
    }
}
