use serde::{Deserialize, Serialize};

use crate::{error::NodeSpecValidationError, renderer_spec::RendererId};

use super::{
    builtin_transformations::BuiltinSpec,
    constraints::{input_count::InputsCountConstraint, Constraints},
    shader::ShaderParam,
    text_spec::TextSpec,
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
    Text(TextSpec),
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
            NodeParams::Builtin { transformation } => Ok(transformation.validate_params(self)?),
            _ => Ok(()),
        }
    }
}

impl NodeParams {
    pub const TEXT_CONSTRAINTS: Constraints = Constraints {
        inputs_count: InputsCountConstraint::Exactly(0),
    };

    pub const IMAGE_CONSTRAINTS: Constraints = Constraints {
        inputs_count: InputsCountConstraint::Exactly(0),
    };
}
