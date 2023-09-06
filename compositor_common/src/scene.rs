use serde::{Deserialize, Serialize};
use std::{fmt::Display, sync::Arc};

use crate::renderer_spec::RendererId;

use self::{
    builtin_transformations::BuiltinSpec,
    text_spec::{TextDimensions, TextSpec},
};

pub mod builtin_transformations;
pub mod text_spec;

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

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeId(pub Arc<str>);

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputId(pub NodeId);

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputId(pub NodeId);

impl Display for InputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0 .0.fmt(f)
    }
}

impl Display for OutputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0 .0.fmt(f)
    }
}

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<NodeId> for InputId {
    fn from(value: NodeId) -> Self {
        Self(value)
    }
}

impl From<NodeId> for OutputId {
    fn from(value: NodeId) -> Self {
        Self(value)
    }
}

/// SceneSpec represents configuration that can be used to create new Scene
/// or update an existing one.
#[derive(Serialize, Deserialize)]
pub struct SceneSpec {
    pub nodes: Vec<NodeSpec>,
    pub outputs: Vec<OutputSpec>,
}

#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
pub struct NodeSpec {
    pub node_id: NodeId,
    #[serde(default)]
    pub input_pads: Vec<NodeId>,
    pub fallback_id: Option<NodeId>,
    #[serde(flatten)]
    pub params: NodeParams,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeParams {
    WebRenderer {
        instance_id: RendererId,
    },
    Shader {
        shader_id: RendererId,
        shader_params: Option<ShaderParam>,
        resolution: Resolution,
    },
    TextRenderer {
        #[serde(flatten)]
        text_params: TextSpec,
        resolution: TextDimensions,
    },
    Image {
        image_id: RendererId,
    },
    #[serde(rename = "built-in")]
    Builtin {
        #[serde(flatten)]
        transformation: BuiltinSpec,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case", content = "value")]
pub enum ShaderParam {
    F32(f32),
    U32(u32),
    I32(i32),
    List(Vec<ShaderParam>),
    Struct(Vec<ShaderParamStructField>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShaderParamStructField {
    pub field_name: String,
    #[serde(flatten)]
    pub value: ShaderParam,
}

impl From<(&'static str, ShaderParam)> for ShaderParamStructField {
    fn from(value: (&'static str, ShaderParam)) -> Self {
        Self {
            field_name: value.0.to_owned(),
            value: value.1,
        }
    }
}
