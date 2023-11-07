use std::time::Duration;

use compositor_common::{renderer_spec, scene};
use compositor_pipeline::pipeline;

use crate::api::UpdateScene;

use super::util::*;
use super::*;

impl From<ComponentId> for scene::NodeId {
    fn from(id: ComponentId) -> Self {
        Self(id.0)
    }
}

impl From<scene::NodeId> for ComponentId {
    fn from(id: scene::NodeId) -> Self {
        Self(id.0)
    }
}

impl From<RendererId> for renderer_spec::RendererId {
    fn from(id: RendererId) -> Self {
        Self(id.0)
    }
}

impl From<renderer_spec::RendererId> for RendererId {
    fn from(id: renderer_spec::RendererId) -> Self {
        Self(id.0)
    }
}

impl From<OutputId> for scene::OutputId {
    fn from(id: OutputId) -> Self {
        Self(scene::NodeId(id.0))
    }
}

impl From<scene::OutputId> for OutputId {
    fn from(id: scene::OutputId) -> Self {
        Self(id.0 .0)
    }
}

impl From<InputId> for scene::InputId {
    fn from(id: InputId) -> Self {
        Self(scene::NodeId(id.0))
    }
}

impl From<scene::InputId> for InputId {
    fn from(id: scene::InputId) -> Self {
        Self(id.0 .0)
    }
}

impl TryFrom<UpdateScene> for scene::SceneSpec {
    type Error = TypeError;

    // very temporary and inefficient
    fn try_from(update_scene: UpdateScene) -> Result<Self, Self::Error> {
        fn try_from_node(node: Component) -> Result<Vec<scene::NodeSpec>, TypeError> {
            if let component::ComponentParams::InputStream(_) = node.params {
                return Ok(vec![]);
            }
            let spec = node.clone().try_into()?;
            let child_specs: Vec<Vec<scene::NodeSpec>> = node
                .children
                .unwrap_or_default()
                .iter()
                .map(|n| try_from_node(n.clone()))
                .collect::<Result<Vec<_>, TypeError>>()?;
            let mut t: Vec<_> = child_specs.into_iter().flatten().collect();
            t.push(spec);
            Ok(t)
        }
        let outputs = update_scene
            .scenes
            .iter()
            .map(|scene| scene::OutputSpec {
                output_id: scene.output_id.clone().into(),
                input_pad: scene.root.id.clone().into(),
            })
            .collect();
        let nodes: Vec<scene::NodeSpec> = update_scene
            .scenes
            .iter()
            .map(|scene| try_from_node(scene.root.clone()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
        let result = Self { nodes, outputs };
        Ok(result)
    }
}

impl TryFrom<InitOptions> for pipeline::Options {
    type Error = TypeError;
    fn try_from(opts: InitOptions) -> Result<Self, Self::Error> {
        let result = Self {
            framerate: opts.framerate.try_into()?,
            stream_fallback_timeout: Duration::from_millis(
                opts.stream_fallback_timeout_ms.unwrap_or(1000.0) as u64,
            ),
            web_renderer: compositor_render::WebRendererOptions {
                init: opts
                    .web_renderer
                    .as_ref()
                    .and_then(|r| r.init)
                    .unwrap_or(true),
                disable_gpu: opts
                    .web_renderer
                    .as_ref()
                    .and_then(|r| r.disable_gpu)
                    .unwrap_or(false),
            },
        };
        Ok(result)
    }
}
