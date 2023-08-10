use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::transformation::TransformationRegistryKey;

use self::text_spec::{TextResolution, TextSpec};

pub mod text_spec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeId(pub Arc<str>);

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputId(pub NodeId);

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputId(pub NodeId);

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
    pub inputs: Vec<InputSpec>,
    pub transforms: Vec<TransformNodeSpec>,
    pub outputs: Vec<OutputSpec>,
}

#[derive(Serialize, Deserialize)]
pub struct InputSpec {
    pub input_id: InputId,
    pub resolution: Resolution,
}

#[derive(Serialize, Deserialize)]
pub struct OutputSpec {
    pub output_id: OutputId,
    pub input_pad: NodeId,
}

#[derive(Serialize, Deserialize)]
pub struct TransformNodeSpec {
    pub node_id: NodeId,
    pub input_pads: Vec<NodeId>,

    #[serde(flatten)]
    pub transform_params: TransformParams,
}

// TODO: tmp clone
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransformParams {
    WebRenderer {
        renderer_id: TransformationRegistryKey,
    },
    Shader {
        shader_id: TransformationRegistryKey,
        shader_params: HashMap<String, ShaderParams>,
        resolution: Resolution,
    },
    TextRenderer {
        text_params: TextSpec,
        resolution: TextResolution,
    },
}

// TODO: tmp clone
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case", content = "value")]
pub enum ShaderParams {
    String(String),
    Binary(Vec<u8>),
}
