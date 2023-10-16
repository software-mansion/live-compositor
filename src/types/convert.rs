use compositor_common::{renderer_spec, scene};

use super::*;

impl From<NodeId> for scene::NodeId {
    fn from(id: NodeId) -> Self {
        Self(id.0)
    }
}

impl From<scene::NodeId> for NodeId {
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

impl TryFrom<Scene> for scene::SceneSpec {
    type Error = anyhow::Error;

    fn try_from(scene: Scene) -> Result<Self, Self::Error> {
        fn from_output(output: Output) -> scene::OutputSpec {
            scene::OutputSpec {
                input_pad: output.input_pad.into(),
                output_id: output.output_id.into(),
            }
        }
        let result = Self {
            nodes: scene
                .nodes
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            outputs: scene.outputs.into_iter().map(from_output).collect(),
        };
        Ok(result)
    }
}

impl From<scene::SceneSpec> for Scene {
    fn from(scene: scene::SceneSpec) -> Self {
        fn from_output(output: scene::OutputSpec) -> Output {
            Output {
                input_pad: output.input_pad.into(),
                output_id: output.output_id.into(),
            }
        }
        Self {
            nodes: scene.nodes.into_iter().map(Into::into).collect(),
            outputs: scene.outputs.into_iter().map(from_output).collect(),
        }
    }
}
