use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::transformation::TransformationRegistryKey;

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
    pub resolution: Resolution,

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
    },
    TextRenderer {
        text_params: TextParams,
    },
}

// TODO: tmp clone
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case", content = "value")]
pub enum ShaderParams {
    String(String),
    Binary(Vec<u8>),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Style {
    Normal,
    Italic,
    Oblique,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub struct Box {
    pub top_left_corner: (u32, u32),
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub struct TextParams {
    pub content: String,
    pub font_size: f32,
    pub placement: Box,
    pub color_rgba: Option<(u8, u8, u8, u8)>,
    pub font_family: Option<String>,
    pub line_height: Option<f32>,
    pub style: Option<Style>,
}
