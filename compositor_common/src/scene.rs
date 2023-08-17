use serde::{Deserialize, Serialize};
use std::{fmt::Display, sync::Arc};

use crate::{transformation::TransformationRegistryKey, util::RGBColor};

use self::{
    common_transformations::CommonTransformation,
    text_spec::{TextDimensions, TextSpec},
};

pub mod common_transformations;
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
    pub inputs: Vec<InputSpec>,
    pub transforms: Vec<TransformNodeSpec>,
    pub outputs: Vec<OutputSpec>,
}

#[derive(Serialize, Deserialize)]
pub struct InputSpec {
    pub input_id: InputId,
    pub resolution: Resolution,
    pub fallback_color_rgb: Option<RGBColor>,
}

#[derive(Serialize, Deserialize)]
pub struct OutputSpec {
    pub output_id: OutputId,
    pub input_pad: NodeId,
}

#[derive(Serialize, Deserialize)]
pub struct TransformNodeSpec {
    pub node_id: NodeId,
    #[serde(default)]
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
        shader_params: Option<ShaderParam>,
        resolution: Resolution,
    },
    TextRenderer {
        text_params: TextSpec,
        resolution: TextDimensions,
    },
    Image {
        image_id: TransformationRegistryKey,
    },
    Common {
        transformation: CommonTransformation,
        resolution: Resolution,
    },
}

// TODO: tmp clone
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
