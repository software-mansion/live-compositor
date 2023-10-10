use serde::{Deserialize, Serialize};

use crate::{error::NodeSpecValidationError, renderer_spec::RendererId};

use super::{
    builtin_transformations::BuiltinSpec,
    constraints::{input_count::InputCountConstraint, Constraint, NodeConstraints},
    shader::ShaderParam,
    text_spec::TextSpec,
    transition::TransitionSpec,
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
    Transition(TransitionSpec),
}

impl NodeSpec {
    pub fn validate_params(&self) -> Result<(), NodeSpecValidationError> {
        match &self.params {
            NodeParams::Builtin { transformation } => Ok(transformation.validate_params(self)?),
            NodeParams::Transition(TransitionSpec { start, end, .. }) => {
                start.validate_params(self)?;
                end.validate_params(self)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

lazy_static! {
    static ref TEXT_CONSTRAINTS: NodeConstraints =
        NodeConstraints(vec![Constraint::InputCount(InputCountConstraint::Exact {
            fixed_count: 0
        })]);
    static ref IMAGE_CONSTRAINTS: NodeConstraints =
        NodeConstraints(vec![Constraint::InputCount(InputCountConstraint::Exact {
            fixed_count: 0
        })]);
}

impl NodeParams {
    pub fn text_constraints() -> &'static NodeConstraints {
        &TEXT_CONSTRAINTS
    }

    pub fn image_constraints() -> &'static NodeConstraints {
        &IMAGE_CONSTRAINTS
    }
}
