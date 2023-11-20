use crate::{error::NodeSpecValidationError, renderer_spec::RendererId};

use super::{
    constraints::{input_count::InputCountConstraint, Constraint, NodeConstraints},
    shader::ShaderParam,
    NodeSpec, Resolution,
};

#[derive(Debug, Clone)]
pub enum NodeParams {
    WebRenderer {
        instance_id: RendererId,
    },
    Shader {
        shader_id: RendererId,
        shader_params: Option<ShaderParam>,
        resolution: Resolution,
    },
    Image {
        image_id: RendererId,
    },
}

impl NodeSpec {
    pub fn validate_params(&self) -> Result<(), NodeSpecValidationError> {
        Ok(())
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
