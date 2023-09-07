use serde::{Deserialize, Serialize};

pub mod builtin_transformations;
pub mod id;
pub mod node;
pub mod shader;
pub mod text_spec;
mod validation;

#[cfg(test)]
mod validation_test;

pub use id::InputId;
pub use id::NodeId;
pub use id::OutputId;
pub use node::NodeParams;

pub const MAX_NODE_RESOLUTION: Resolution = Resolution {
    width: 7682,
    height: 4320,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

impl Resolution {
    pub fn ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

/// SceneSpec represents configuration that can be used to create new Scene
/// or update an existing one.
#[derive(Debug, Serialize, Deserialize)]
pub struct SceneSpec {
    pub nodes: Vec<NodeSpec>,
    pub outputs: Vec<OutputSpec>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputSpec {
    pub output_id: OutputId,
    pub input_pad: NodeId,
}

/// NodeSpec provides a configuration necessary to construct Node. Node is a core
/// abstraction in the rendering pipeline, it represents a logic that transforms
/// zero or more inputs into an output stream. Most nodes are wrappers over the
/// renderers and just a way to provide parameters to them.
///
/// Distinction whether logic should be part of the Node or Renderer should be based
/// on how long the initialization is. Heavy operations should be part of renderer and
/// light ones part of the Node. Simple rule of thumb for what is heavy/light is answer
/// to the question: Would it still work if it's done every frame.
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeSpec {
    pub node_id: NodeId,
    #[serde(default)]
    pub input_pads: Vec<NodeId>,
    pub fallback_id: Option<NodeId>,
    #[serde(flatten)]
    pub params: NodeParams,
}
