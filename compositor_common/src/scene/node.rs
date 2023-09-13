use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{error::NodeSpecValidationError, renderer_spec::RendererId};

use super::{
    builtin_transformations::BuiltinSpec,
    shader::ShaderParam,
    text_spec::{TextDimensions, TextSpec},
    NodeSpec, Resolution,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl NodeSpec {
    pub fn validate_params(&self) -> Result<(), NodeSpecValidationError> {
        match &self.params {
            NodeParams::Builtin { transformation, .. } => Ok(transformation.validate_params(self)?),
            _ => Ok(()),
        }
    }

    pub fn transformation_name(&self) -> Arc<str> {
        match &self.params {
            NodeParams::WebRenderer { instance_id } => instance_id.0.clone(),
            NodeParams::Shader { shader_id, .. } => shader_id.0.clone(),
            NodeParams::TextRenderer { .. } => Arc::from("text_renderer"),
            NodeParams::Image { image_id } => image_id.0.clone(),
            NodeParams::Builtin { transformation } => transformation.transformation_name(),
        }
    }
}
